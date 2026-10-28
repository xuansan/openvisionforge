use std::{
    collections::BTreeMap,
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    dag::topo_order, validate_workflow, NodeTrace, OperatorRegistry, PortTrace, ProcessingMetrics,
    Result, RunCondition, RunEvent, RunMetrics, RunTrace, Value, VisionError, Workflow,
    WorkflowNode,
};

pub struct Executor<'a> {
    registry: &'a OperatorRegistry,
}

pub struct ExecutionContext {
    pub run_id: String,
    pub records_dir: PathBuf,
}

pub struct NodeContext<'a> {
    pub node: &'a WorkflowNode,
    pub inputs: BTreeMap<String, Value>,
    pub execution: &'a ExecutionContext,
}

impl<'a> Executor<'a> {
    pub fn new(registry: &'a OperatorRegistry) -> Self {
        Self { registry }
    }

    pub fn run(&self, workflow: &Workflow, records_dir: impl AsRef<Path>) -> Result<RunTrace> {
        self.validate(workflow)?;
        let validation = validate_workflow(workflow, self.registry);
        if !validation.ok {
            return Err(VisionError::Operator {
                node: "workflow".to_string(),
                message: validation
                    .issues
                    .iter()
                    .map(|issue| issue.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
            });
        }

        let run_id = new_run_id();
        let execution = ExecutionContext {
            run_id: run_id.clone(),
            records_dir: records_dir.as_ref().join(&run_id),
        };
        fs::create_dir_all(&execution.records_dir)?;

        let order = topo_order(workflow)?;
        let nodes = workflow
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        let mut values = BTreeMap::<(String, String), Value>::new();
        let mut traces = Vec::new();
        let run_start = Instant::now();
        let mut processed_frames = 0_u64;
        let mut image_outputs = 0_u64;
        let mut short_circuited = false;

        for node_id in order {
            let node = nodes.get(node_id.as_str()).expect("topo node exists");
            let schema = self.registry.get(&node.operator)?;
            let Some(inputs) = collect_inputs(workflow, &values, &node.id, schema.inputs) else {
                traces.push(skipped_trace(node, "upstream_skipped"));
                continue;
            };
            if short_circuited && !node.execution.always_run {
                traces.push(skipped_trace(node, "short_circuit"));
                continue;
            }
            if !condition_matches(node.execution.run_when, &inputs) {
                traces.push(skipped_trace(node, "condition_not_met"));
                continue;
            }
            let input_trace = port_trace(&inputs);
            let node_start = Instant::now();
            let context = NodeContext {
                node,
                inputs,
                execution: &execution,
            };
            let outputs = match (schema.run)(&context) {
                Ok(outputs) => outputs,
                Err(err) => {
                    let node_duration_ms = node_start.elapsed().as_secs_f64() * 1000.0;
                    let message = err.to_string();
                    let result = if message.to_ascii_lowercase().contains("timeout") {
                        "TIMEOUT"
                    } else {
                        "ERROR"
                    };
                    traces.push(NodeTrace {
                        node_id: node.id.clone(),
                        operator: node.operator.clone(),
                        inputs: input_trace,
                        outputs: Vec::new(),
                        status: format!("error:{message}"),
                        metrics: ProcessingMetrics {
                            duration_ms: round3(node_duration_ms),
                            input_images: image_count(context.inputs.values()),
                            output_images: 0,
                        },
                    });
                    let total_duration_ms = run_start.elapsed().as_secs_f64() * 1000.0;
                    let metrics = RunMetrics {
                        total_duration_ms: round3(total_duration_ms),
                        processed_frames,
                        image_outputs,
                        fps: fps(processed_frames, total_duration_ms),
                    };
                    let trace = RunTrace {
                        run_id: run_id.clone(),
                        workflow_id: workflow.id.clone(),
                        nodes: traces,
                        metrics: metrics.clone(),
                    };
                    let event = RunEvent {
                        run_id: run_id.clone(),
                        workflow_id: workflow.id.clone(),
                        status: "failed".to_string(),
                        result: result.to_string(),
                        metrics,
                    };
                    write_run_files(workflow, &execution, &trace, &event, &values)?;
                    return Err(VisionError::Operator {
                        node: node.id.clone(),
                        message,
                    });
                }
            };
            let node_duration_ms = node_start.elapsed().as_secs_f64() * 1000.0;

            for required in schema.outputs {
                if !outputs.contains_key(*required) {
                    let message = format!("missing required output {}.{}", node.id, required);
                    traces.push(NodeTrace {
                        node_id: node.id.clone(),
                        operator: node.operator.clone(),
                        inputs: input_trace.clone(),
                        outputs: port_trace(&outputs),
                        status: format!("error:{message}"),
                        metrics: ProcessingMetrics {
                            duration_ms: round3(node_duration_ms),
                            input_images: image_count(context.inputs.values()),
                            output_images: image_count(outputs.values()),
                        },
                    });
                    let total_duration_ms = run_start.elapsed().as_secs_f64() * 1000.0;
                    let metrics = RunMetrics {
                        total_duration_ms: round3(total_duration_ms),
                        processed_frames,
                        image_outputs,
                        fps: fps(processed_frames, total_duration_ms),
                    };
                    let trace = RunTrace {
                        run_id: run_id.clone(),
                        workflow_id: workflow.id.clone(),
                        nodes: traces,
                        metrics: metrics.clone(),
                    };
                    let event = RunEvent {
                        run_id: run_id.clone(),
                        workflow_id: workflow.id.clone(),
                        status: "failed".to_string(),
                        result: "ERROR".to_string(),
                        metrics,
                    };
                    write_run_files(workflow, &execution, &trace, &event, &values)?;
                    return Err(VisionError::MissingOutput {
                        node: node.id.clone(),
                        port: required.to_string(),
                    });
                }
            }

            let output_trace = port_trace(&outputs);
            let input_images = image_count(context.inputs.values());
            let output_images = image_count(outputs.values());
            image_outputs += output_images;
            if produced_source_frame(&context.inputs, &outputs) {
                processed_frames += output_images;
            }
            for (port, value) in outputs {
                values.insert((node.id.clone(), port), value);
            }
            if node.execution.short_circuit_on_ng
                && output_decisions(&values, &node.id)
                    .iter()
                    .any(|decision| !decision)
            {
                short_circuited = true;
            }
            traces.push(NodeTrace {
                node_id: node.id.clone(),
                operator: node.operator.clone(),
                inputs: input_trace,
                outputs: output_trace,
                status: "ok".to_string(),
                metrics: ProcessingMetrics {
                    duration_ms: round3(node_duration_ms),
                    input_images,
                    output_images,
                },
            });
        }

        let total_duration_ms = run_start.elapsed().as_secs_f64() * 1000.0;
        let metrics = RunMetrics {
            total_duration_ms: round3(total_duration_ms),
            processed_frames,
            image_outputs,
            fps: fps(processed_frames, total_duration_ms),
        };
        let trace = RunTrace {
            run_id: run_id.clone(),
            workflow_id: workflow.id.clone(),
            nodes: traces,
            metrics: metrics.clone(),
        };
        let event = RunEvent {
            run_id,
            workflow_id: workflow.id.clone(),
            status: "finished".to_string(),
            result: final_result(workflow, &values),
            metrics,
        };
        write_run_files(workflow, &execution, &trace, &event, &values)?;
        Ok(trace)
    }

    fn validate(&self, workflow: &Workflow) -> Result<()> {
        let mut nodes = BTreeMap::new();
        for node in &workflow.nodes {
            if nodes.insert(node.id.as_str(), node).is_some() {
                return Err(VisionError::DuplicateNode(node.id.clone()));
            }
            self.registry.get(&node.operator)?;
        }

        let mut target_ports = BTreeSet::new();
        for edge in &workflow.edges {
            let from = nodes
                .get(edge.from_node.as_str())
                .ok_or_else(|| VisionError::UnknownNode(edge.from_node.clone()))?;
            let to = nodes
                .get(edge.to_node.as_str())
                .ok_or_else(|| VisionError::UnknownNode(edge.to_node.clone()))?;
            let from_schema = self.registry.get(&from.operator)?;
            let to_schema = self.registry.get(&to.operator)?;

            if !from_schema.outputs.contains(&edge.from_port.as_str()) {
                return Err(VisionError::UnknownPort {
                    operator: from.operator.clone(),
                    port: edge.from_port.clone(),
                    direction: "output",
                });
            }
            if !to_schema.inputs.contains(&edge.to_port.as_str())
                && !to_schema.optional_inputs.contains(&edge.to_port.as_str())
            {
                return Err(VisionError::UnknownPort {
                    operator: to.operator.clone(),
                    port: edge.to_port.clone(),
                    direction: "input",
                });
            }

            let target_key = (edge.to_node.as_str(), edge.to_port.as_str());
            if !target_ports.insert(target_key) {
                return Err(VisionError::InputPortAlreadyConnected {
                    node: edge.to_node.clone(),
                    port: edge.to_port.clone(),
                });
            }
        }

        for node in &workflow.nodes {
            let schema = self.registry.get(&node.operator)?;
            let connected = workflow
                .edges
                .iter()
                .filter(|edge| edge.to_node == node.id)
                .map(|edge| edge.to_port.as_str())
                .collect::<BTreeSet<_>>();
            for required in schema.inputs {
                if !connected.contains(required) {
                    return Err(VisionError::MissingInput {
                        node: node.id.clone(),
                        port: required.to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

fn produced_source_frame(
    inputs: &BTreeMap<String, Value>,
    outputs: &BTreeMap<String, Value>,
) -> bool {
    !inputs
        .values()
        .any(|value| matches!(value, Value::Image(_)))
        && outputs
            .values()
            .any(|value| matches!(value, Value::Image(_)))
}

fn image_count<'a>(values: impl Iterator<Item = &'a Value>) -> u64 {
    values
        .filter(|value| matches!(value, Value::Image(_)))
        .count() as u64
}

fn fps(processed_frames: u64, total_duration_ms: f64) -> f64 {
    if processed_frames == 0 || total_duration_ms <= 0.0 {
        0.0
    } else {
        round3(processed_frames as f64 / (total_duration_ms / 1000.0))
    }
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn collect_inputs(
    workflow: &Workflow,
    values: &BTreeMap<(String, String), Value>,
    node_id: &str,
    required_ports: &[&str],
) -> Option<BTreeMap<String, Value>> {
    let mut inputs = BTreeMap::new();
    for edge in workflow.edges.iter().filter(|edge| edge.to_node == node_id) {
        if let Some(value) = values.get(&(edge.from_node.clone(), edge.from_port.clone())) {
            inputs.insert(edge.to_port.clone(), value.clone());
        } else if required_ports.contains(&edge.to_port.as_str()) {
            return None;
        }
    }
    Some(inputs)
}

fn condition_matches(condition: RunCondition, inputs: &BTreeMap<String, Value>) -> bool {
    if condition == RunCondition::Always {
        return true;
    }
    let decisions = input_decisions(inputs);
    if decisions.is_empty() {
        return false;
    }
    match condition {
        RunCondition::Always => true,
        RunCondition::AllOk => decisions.iter().all(|value| *value),
        RunCondition::AnyOk => decisions.iter().any(|value| *value),
        RunCondition::AllNg => decisions.iter().all(|value| !*value),
        RunCondition::AnyNg => decisions.iter().any(|value| !*value),
    }
}

fn input_decisions(inputs: &BTreeMap<String, Value>) -> Vec<bool> {
    inputs.values().filter_map(decision).collect()
}

fn output_decisions(values: &BTreeMap<(String, String), Value>, node_id: &str) -> Vec<bool> {
    values
        .iter()
        .filter(|((owner, _), _)| owner == node_id)
        .filter_map(|(_, value)| decision(value))
        .collect()
}

fn decision(value: &Value) -> Option<bool> {
    match value {
        Value::Rule(rule) => Some(rule.ok),
        Value::Rules(rules) => Some(rules.iter().all(|rule| rule.ok)),
        Value::Final(result) => Some(result.final_ok),
        Value::Bool(value) => Some(*value),
        _ => None,
    }
}

fn skipped_trace(node: &WorkflowNode, reason: &str) -> NodeTrace {
    NodeTrace {
        node_id: node.id.clone(),
        operator: node.operator.clone(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        status: format!("skipped:{reason}"),
        metrics: ProcessingMetrics {
            duration_ms: 0.0,
            input_images: 0,
            output_images: 0,
        },
    }
}

fn write_inspection_result(
    workflow: &Workflow,
    execution: &ExecutionContext,
    trace: &RunTrace,
    event: &RunEvent,
    values: &BTreeMap<(String, String), Value>,
) -> Result<()> {
    let outputs = values
        .iter()
        .map(|((node_id, port), value)| {
            serde_json::json!({ "node_id": node_id, "port": port, "value": value })
        })
        .collect::<Vec<_>>();
    let final_value = values.values().find_map(|value| match value {
        Value::Final(result) => Some(result),
        _ => None,
    });
    let result = serde_json::json!({
        "schema_version": "1.0",
        "run_id": execution.run_id,
        "workflow": {
            "id": workflow.id,
            "name": workflow.name,
            "version": workflow.version,
            "metadata": workflow.metadata,
        },
        "status": event.result,
        "final_result": final_value,
        "metrics": trace.metrics,
        "nodes": trace.nodes,
        "outputs": outputs,
    });
    fs::write(
        execution.records_dir.join("inspection_result.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(())
}

fn write_run_files(
    workflow: &Workflow,
    execution: &ExecutionContext,
    trace: &RunTrace,
    event: &RunEvent,
    values: &BTreeMap<(String, String), Value>,
) -> Result<()> {
    fs::write(
        execution.records_dir.join("trace.json"),
        serde_json::to_vec_pretty(trace)?,
    )?;
    fs::write(
        execution.records_dir.join("event.json"),
        serde_json::to_vec_pretty(event)?,
    )?;
    write_inspection_result(workflow, execution, trace, event, values)
}

fn final_result(workflow: &Workflow, values: &BTreeMap<(String, String), Value>) -> String {
    let final_nodes = workflow
        .nodes
        .iter()
        .filter(|node| node.operator == "logic.final_result")
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    if let [node_id] = final_nodes.as_slice() {
        if let Some(Value::Final(result)) = values.get(&(node_id.to_string(), "final".to_string()))
        {
            return result.status.clone();
        }
    }
    values
        .values()
        .find_map(|value| match value {
            Value::Final(result) => Some(result.status.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "UNKNOWN".to_string())
}

fn port_trace(values: &BTreeMap<String, Value>) -> Vec<PortTrace> {
    values
        .iter()
        .map(|(port, value)| PortTrace {
            port: port.clone(),
            value: value.summary(),
        })
        .collect()
}

fn new_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    format!("run_{nanos}")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        NodeExecutionPolicy, OperatorSchema, RunCondition, Workflow, WorkflowEdge, WorkflowNode,
    };

    use super::*;

    #[test]
    fn rejects_missing_required_input() {
        let mut registry = OperatorRegistry::new();
        registry.register(OperatorSchema {
            id: "needs.image",
            name: "Needs Image",
            category: "Test",
            inputs: &["image"],
            optional_inputs: &[],
            outputs: &["ok"],
            params: &[],
            run: |_| Ok(BTreeMap::from([("ok".to_string(), Value::Bool(true))])),
        });
        let workflow = Workflow {
            id: "w".to_string(),
            name: "w".to_string(),
            version: "0".to_string(),
            metadata: Default::default(),
            nodes: vec![WorkflowNode {
                id: "n1".to_string(),
                operator: "needs.image".to_string(),
                params: serde_json::Value::Null,
                execution: Default::default(),
            }],
            edges: vec![],
        };

        assert!(matches!(
            Executor::new(&registry).run(&workflow, std::env::temp_dir()),
            Err(VisionError::MissingInput { .. })
        ));
    }

    #[test]
    fn rejects_multiple_sources_for_one_input_port() {
        let mut registry = OperatorRegistry::new();
        registry.register(OperatorSchema {
            id: "source.image",
            name: "Source Image",
            category: "Test",
            inputs: &[],
            optional_inputs: &[],
            outputs: &["image"],
            params: &[],
            run: |_| {
                Ok(BTreeMap::from([(
                    "image".to_string(),
                    Value::Image(crate::ImageFrame {
                        width: 1,
                        height: 1,
                        channels: 1,
                        pixel_format: "gray8".to_string(),
                        source_uri: Some("mock.png".to_string()),
                    }),
                )]))
            },
        });
        registry.register(OperatorSchema {
            id: "needs.image",
            name: "Needs Image",
            category: "Test",
            inputs: &["image"],
            optional_inputs: &[],
            outputs: &["ok"],
            params: &[],
            run: |_| Ok(BTreeMap::from([("ok".to_string(), Value::Bool(true))])),
        });
        let workflow = Workflow {
            id: "w".to_string(),
            name: "w".to_string(),
            version: "0".to_string(),
            metadata: Default::default(),
            nodes: vec![
                node("a", "source.image"),
                node("b", "source.image"),
                node("c", "needs.image"),
            ],
            edges: vec![
                edge("a", "image", "c", "image"),
                edge("b", "image", "c", "image"),
            ],
        };

        assert!(matches!(
            Executor::new(&registry).run(&workflow, std::env::temp_dir()),
            Err(VisionError::InputPortAlreadyConnected { .. })
        ));
    }

    #[test]
    fn short_circuit_skips_normal_nodes_but_runs_always_nodes() {
        let mut registry = OperatorRegistry::new();
        registry.register(OperatorSchema {
            id: "source.false",
            name: "False",
            category: "Test",
            inputs: &[],
            optional_inputs: &[],
            outputs: &["passed"],
            params: &[],
            run: |_| Ok(BTreeMap::from([("passed".to_string(), Value::Bool(false))])),
        });
        registry.register(OperatorSchema {
            id: "sink",
            name: "Sink",
            category: "Test",
            inputs: &["passed"],
            optional_inputs: &[],
            outputs: &["ok"],
            params: &[],
            run: |_| Ok(BTreeMap::from([("ok".to_string(), Value::Bool(true))])),
        });
        let mut source = node("source", "source.false");
        source.execution.short_circuit_on_ng = true;
        let skipped = node("skipped", "sink");
        let mut always = node("always", "sink");
        always.execution.always_run = true;
        let workflow = Workflow {
            id: "short".to_string(),
            name: "short".to_string(),
            version: "0".to_string(),
            metadata: Default::default(),
            nodes: vec![source, skipped, always],
            edges: vec![
                edge("source", "passed", "skipped", "passed"),
                edge("source", "passed", "always", "passed"),
            ],
        };

        let trace = Executor::new(&registry)
            .run(&workflow, std::env::temp_dir())
            .unwrap();

        assert_eq!(trace.nodes[0].status, "ok");
        assert!(trace
            .nodes
            .iter()
            .any(|node| node.node_id == "skipped" && node.status == "skipped:short_circuit"));
        assert!(trace
            .nodes
            .iter()
            .any(|node| node.node_id == "always" && node.status == "ok"));
    }

    #[test]
    fn run_condition_skips_when_decision_does_not_match() {
        let mut registry = OperatorRegistry::new();
        registry.register(OperatorSchema {
            id: "source.false",
            name: "False",
            category: "Test",
            inputs: &[],
            optional_inputs: &[],
            outputs: &["passed"],
            params: &[],
            run: |_| Ok(BTreeMap::from([("passed".to_string(), Value::Bool(false))])),
        });
        registry.register(OperatorSchema {
            id: "sink",
            name: "Sink",
            category: "Test",
            inputs: &["passed"],
            optional_inputs: &[],
            outputs: &["ok"],
            params: &[],
            run: |_| Ok(BTreeMap::from([("ok".to_string(), Value::Bool(true))])),
        });
        let source = node("source", "source.false");
        let mut conditional = node("conditional", "sink");
        conditional.execution.run_when = RunCondition::AllOk;
        let workflow = Workflow {
            id: "condition".to_string(),
            name: "condition".to_string(),
            version: "0".to_string(),
            metadata: Default::default(),
            nodes: vec![source, conditional],
            edges: vec![edge("source", "passed", "conditional", "passed")],
        };

        let trace = Executor::new(&registry)
            .run(&workflow, std::env::temp_dir())
            .unwrap();

        assert!(trace.nodes.iter().any(|node| {
            node.node_id == "conditional" && node.status == "skipped:condition_not_met"
        }));
    }

    #[test]
    fn operator_error_writes_error_event_and_trace() {
        let mut registry = OperatorRegistry::new();
        registry.register(OperatorSchema {
            id: "source.error",
            name: "Error",
            category: "Test",
            inputs: &[],
            optional_inputs: &[],
            outputs: &["ok"],
            params: &[],
            run: |_| {
                Err(VisionError::Operator {
                    node: "source".to_string(),
                    message: "test failure".to_string(),
                })
            },
        });
        let workflow = Workflow {
            id: "error".to_string(),
            name: "error".to_string(),
            version: "0".to_string(),
            metadata: Default::default(),
            nodes: vec![node("source", "source.error")],
            edges: vec![],
        };
        let root = std::env::temp_dir().join(new_run_id());

        assert!(Executor::new(&registry).run(&workflow, &root).is_err());

        let run_dir = std::fs::read_dir(&root)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let event: serde_json::Value =
            serde_json::from_slice(&std::fs::read(run_dir.join("event.json")).unwrap()).unwrap();
        assert_eq!(event["result"], "ERROR");
        assert!(run_dir.join("trace.json").exists());
        assert!(run_dir.join("inspection_result.json").exists());
    }

    #[test]
    fn missing_output_writes_error_event_and_trace() {
        let mut registry = OperatorRegistry::new();
        registry.register(OperatorSchema {
            id: "source.missing",
            name: "Missing",
            category: "Test",
            inputs: &[],
            optional_inputs: &[],
            outputs: &["ok"],
            params: &[],
            run: |_| Ok(BTreeMap::new()),
        });
        let workflow = Workflow {
            id: "missing".to_string(),
            name: "missing".to_string(),
            version: "0".to_string(),
            metadata: Default::default(),
            nodes: vec![node("source", "source.missing")],
            edges: vec![],
        };
        let root = std::env::temp_dir().join(new_run_id());

        assert!(matches!(
            Executor::new(&registry).run(&workflow, &root),
            Err(VisionError::MissingOutput { .. })
        ));

        let run_dir = std::fs::read_dir(&root)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let event: serde_json::Value =
            serde_json::from_slice(&std::fs::read(run_dir.join("event.json")).unwrap()).unwrap();
        assert_eq!(event["result"], "ERROR");
        assert!(run_dir.join("trace.json").exists());
        assert!(run_dir.join("inspection_result.json").exists());
    }

    #[test]
    fn executor_applies_shared_port_type_validation() {
        let mut registry = OperatorRegistry::new();
        registry.register(OperatorSchema {
            id: "source.image",
            name: "Image",
            category: "Test",
            inputs: &[],
            optional_inputs: &[],
            outputs: &["image"],
            params: &[],
            run: |_| Ok(BTreeMap::new()),
        });
        registry.register(OperatorSchema {
            id: "sink.bool",
            name: "Bool",
            category: "Test",
            inputs: &["passed"],
            optional_inputs: &[],
            outputs: &["ok"],
            params: &[],
            run: |_| Ok(BTreeMap::new()),
        });
        let workflow = Workflow {
            id: "types".to_string(),
            name: "types".to_string(),
            version: "0".to_string(),
            metadata: Default::default(),
            nodes: vec![node("source", "source.image"), node("sink", "sink.bool")],
            edges: vec![edge("source", "image", "sink", "passed")],
        };

        let err = Executor::new(&registry)
            .run(&workflow, std::env::temp_dir())
            .expect_err("image to bool must fail shared validation");

        assert!(err.to_string().contains("type mismatch"));
    }

    fn node(id: &str, operator: &str) -> WorkflowNode {
        WorkflowNode {
            id: id.to_string(),
            operator: operator.to_string(),
            params: serde_json::Value::Null,
            execution: NodeExecutionPolicy::default(),
        }
    }

    fn edge(from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> WorkflowEdge {
        WorkflowEdge {
            from_node: from_node.to_string(),
            from_port: from_port.to_string(),
            to_node: to_node.to_string(),
            to_port: to_port.to_string(),
        }
    }
}
