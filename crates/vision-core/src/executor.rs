use std::{
    collections::BTreeMap,
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    dag::topo_order, NodeTrace, OperatorRegistry, PortTrace, ProcessingMetrics, Result, RunEvent,
    RunMetrics, RunTrace, Value, VisionError, Workflow, WorkflowNode,
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

        for node_id in order {
            let node = nodes.get(node_id.as_str()).expect("topo node exists");
            let schema = self.registry.get(&node.operator)?;
            let inputs = collect_inputs(workflow, &values, &node.id)?;
            let input_trace = port_trace(&inputs);
            let node_start = Instant::now();
            let context = NodeContext {
                node,
                inputs,
                execution: &execution,
            };
            let outputs = (schema.run)(&context).map_err(|err| VisionError::Operator {
                node: node.id.clone(),
                message: err.to_string(),
            })?;
            let node_duration_ms = node_start.elapsed().as_secs_f64() * 1000.0;

            for required in schema.outputs {
                if !outputs.contains_key(*required) {
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
            result: final_result(&values),
            metrics,
        };
        fs::write(
            execution.records_dir.join("trace.json"),
            serde_json::to_vec_pretty(&trace)?,
        )?;
        fs::write(
            execution.records_dir.join("event.json"),
            serde_json::to_vec_pretty(&event)?,
        )?;
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
) -> Result<BTreeMap<String, Value>> {
    let mut inputs = BTreeMap::new();
    for edge in workflow.edges.iter().filter(|edge| edge.to_node == node_id) {
        let value = values
            .get(&(edge.from_node.clone(), edge.from_port.clone()))
            .ok_or_else(|| VisionError::MissingInput {
                node: edge.to_node.clone(),
                port: edge.to_port.clone(),
            })?;
        inputs.insert(edge.to_port.clone(), value.clone());
    }
    Ok(inputs)
}

fn final_result(values: &BTreeMap<(String, String), Value>) -> String {
    values
        .iter()
        .find_map(|((_, port), value)| match (port.as_str(), value) {
            ("status", Value::Text(status)) => Some(status.clone()),
            ("final", Value::Final(result)) => Some(result.status.clone()),
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

    use crate::{OperatorSchema, Workflow, WorkflowEdge, WorkflowNode};

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
            nodes: vec![WorkflowNode {
                id: "n1".to_string(),
                operator: "needs.image".to_string(),
                params: serde_json::Value::Null,
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

    fn node(id: &str, operator: &str) -> WorkflowNode {
        WorkflowNode {
            id: id.to_string(),
            operator: operator.to_string(),
            params: serde_json::Value::Null,
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
