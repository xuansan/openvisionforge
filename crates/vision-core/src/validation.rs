use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{dag::topo_order, OperatorRegistry, ParamKind, Workflow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortValueType {
    Image,
    Mask,
    Regions,
    Contours,
    Measurements,
    Location,
    Detections,
    Classifications,
    Segmentations,
    Poses,
    OrientedBoxes,
    Rule,
    Final,
    Trigger,
    Capture,
    Signal,
    Bool,
    Text,
    Json,
    Any,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortSchema {
    pub name: String,
    pub value_type: PortValueType,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub level: ValidationLevel,
    pub code: &'static str,
    pub message: String,
    pub node_id: Option<String>,
    pub edge_id: Option<String>,
    pub port: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub ok: bool,
    pub nodes: usize,
    pub edges: usize,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    fn new(workflow: &Workflow) -> Self {
        Self {
            ok: true,
            nodes: workflow.nodes.len(),
            edges: workflow.edges.len(),
            issues: Vec::new(),
        }
    }

    fn push(&mut self, issue: ValidationIssue) {
        if issue.level == ValidationLevel::Error {
            self.ok = false;
        }
        self.issues.push(issue);
    }
}

pub fn validate_workflow(workflow: &Workflow, registry: &OperatorRegistry) -> ValidationReport {
    let mut report = ValidationReport::new(workflow);

    if let Err(err) = topo_order(workflow) {
        report.push(error("cycle", err.to_string(), None, None, None));
    }

    let mut nodes = BTreeMap::new();
    for node in &workflow.nodes {
        if nodes.insert(node.id.as_str(), node).is_some() {
            report.push(error(
                "duplicate_node",
                format!("duplicate node id: {}", node.id),
                Some(node.id.clone()),
                None,
                None,
            ));
        }
        if let Err(err) = registry.get(&node.operator) {
            report.push(error(
                "unknown_operator",
                err.to_string(),
                Some(node.id.clone()),
                None,
                None,
            ));
        }
    }

    let mut edge_keys = BTreeSet::new();
    let mut target_ports = BTreeSet::new();
    for edge in &workflow.edges {
        let edge_id = edge_id(
            &edge.from_node,
            &edge.from_port,
            &edge.to_node,
            &edge.to_port,
        );
        if !edge_keys.insert(edge_id.clone()) {
            report.push(error(
                "duplicate_edge",
                format!("duplicate edge: {edge_id}"),
                None,
                Some(edge_id.clone()),
                None,
            ));
            continue;
        }
        let target_key = format!("{}:{}", edge.to_node, edge.to_port);
        if !target_ports.insert(target_key.clone()) {
            report.push(error(
                "input_port_already_connected",
                format!("input port already connected: {target_key}"),
                Some(edge.to_node.clone()),
                Some(edge_id.clone()),
                Some(edge.to_port.clone()),
            ));
            continue;
        }
        let Some(from) = nodes.get(edge.from_node.as_str()) else {
            report.push(error(
                "unknown_from_node",
                format!("unknown from_node: {}", edge.from_node),
                None,
                Some(edge_id),
                None,
            ));
            continue;
        };
        let Some(to) = nodes.get(edge.to_node.as_str()) else {
            report.push(error(
                "unknown_to_node",
                format!("unknown to_node: {}", edge.to_node),
                None,
                Some(edge_id),
                None,
            ));
            continue;
        };
        let Ok(from_schema) = registry.get(&from.operator) else {
            continue;
        };
        let Ok(to_schema) = registry.get(&to.operator) else {
            continue;
        };

        if !from_schema.outputs.contains(&edge.from_port.as_str()) {
            report.push(error(
                "unknown_output_port",
                format!("{} has no output port {}", from.id, edge.from_port),
                Some(from.id.clone()),
                Some(edge_id.clone()),
                Some(edge.from_port.clone()),
            ));
            continue;
        }
        if !to_schema.inputs.contains(&edge.to_port.as_str())
            && !to_schema.optional_inputs.contains(&edge.to_port.as_str())
        {
            report.push(error(
                "unknown_input_port",
                format!("{} has no input port {}", to.id, edge.to_port),
                Some(to.id.clone()),
                Some(edge_id.clone()),
                Some(edge.to_port.clone()),
            ));
            continue;
        }

        let output_type = infer_port_type(&edge.from_port);
        let input_type = infer_port_type(&edge.to_port);
        if !types_compatible(output_type, input_type) {
            report.push(error(
                "port_type_mismatch",
                format!(
                    "type mismatch: {}.{} ({:?}) cannot connect to {}.{} ({:?})",
                    from.id, edge.from_port, output_type, to.id, edge.to_port, input_type
                ),
                Some(to.id.clone()),
                Some(edge_id),
                Some(edge.to_port.clone()),
            ));
        }
    }

    for node in &workflow.nodes {
        let Ok(schema) = registry.get(&node.operator) else {
            continue;
        };
        let connected = workflow
            .edges
            .iter()
            .filter(|edge| edge.to_node == node.id)
            .map(|edge| edge.to_port.as_str())
            .collect::<BTreeSet<_>>();
        for required in schema.inputs {
            if !connected.contains(required) {
                report.push(error(
                    "missing_required_input",
                    format!("{} missing required input {}", node.id, required),
                    Some(node.id.clone()),
                    None,
                    Some(required.to_string()),
                ));
            }
        }

        for param in schema.params.iter().filter(|param| param.required) {
            let missing = node
                .params
                .get(param.key)
                .map(|value| value.is_null() || value.as_str().is_some_and(str::is_empty))
                .unwrap_or(true);
            if missing {
                report.push(error(
                    "missing_required_param",
                    format!("{} missing required param {}", node.id, param.key),
                    Some(node.id.clone()),
                    None,
                    Some(param.key.to_string()),
                ));
            }
        }

        for param in schema.params {
            if let Some(value) = node.params.get(param.key) {
                if !param_value_compatible(param.kind, value) {
                    report.push(error(
                        "param_type_mismatch",
                        format!(
                            "{} param {} does not match {:?}",
                            node.id, param.key, param.kind
                        ),
                        Some(node.id.clone()),
                        None,
                        Some(param.key.to_string()),
                    ));
                }
                if !param.options.is_empty() {
                    let text = value.as_str().unwrap_or_default();
                    if !text.is_empty() && !param.options.contains(&text) {
                        report.push(error(
                            "param_option_invalid",
                            format!(
                                "{} param {} must be one of {:?}",
                                node.id, param.key, param.options
                            ),
                            Some(node.id.clone()),
                            None,
                            Some(param.key.to_string()),
                        ));
                    }
                }
                if let Some(number) = value.as_f64() {
                    if param.min.is_some_and(|min| number < min) {
                        report.push(error(
                            "param_below_min",
                            format!(
                                "{} param {} below min {}",
                                node.id,
                                param.key,
                                param.min.unwrap()
                            ),
                            Some(node.id.clone()),
                            None,
                            Some(param.key.to_string()),
                        ));
                    }
                    if param.max.is_some_and(|max| number > max) {
                        report.push(error(
                            "param_above_max",
                            format!(
                                "{} param {} above max {}",
                                node.id,
                                param.key,
                                param.max.unwrap()
                            ),
                            Some(node.id.clone()),
                            None,
                            Some(param.key.to_string()),
                        ));
                    }
                }
            }
        }
    }

    report
}

pub fn port_schema(name: &str, required: bool) -> PortSchema {
    PortSchema {
        name: name.to_string(),
        value_type: infer_port_type(name),
        required,
    }
}

pub fn infer_port_type(name: &str) -> PortValueType {
    if name.starts_with("rule") {
        return PortValueType::Rule;
    }
    match name {
        "image" | "gray" | "frame" | "annotated" => PortValueType::Image,
        "mask" => PortValueType::Mask,
        "regions" | "blobs" => PortValueType::Regions,
        "contours" => PortValueType::Contours,
        "measurements" | "measurement" | "metrics" => PortValueType::Measurements,
        "location" | "pose" => PortValueType::Location,
        "detections" => PortValueType::Detections,
        "classes" | "classifications" => PortValueType::Classifications,
        "masks" | "segmentations" => PortValueType::Segmentations,
        "poses" | "keypoints" => PortValueType::Poses,
        "oriented_boxes" | "obb" => PortValueType::OrientedBoxes,
        "rules" => PortValueType::Rule,
        "final" => PortValueType::Final,
        "trigger" => PortValueType::Trigger,
        "capture" => PortValueType::Capture,
        "signal" => PortValueType::Signal,
        "ok" | "passed" => PortValueType::Bool,
        "text" | "message" | "status" => PortValueType::Text,
        "raw" | "stats" | "payload" | "files" | "record" | "trace" | "result" | "failed_count"
        | "failed_regions" => PortValueType::Json,
        _ => PortValueType::Any,
    }
}

fn types_compatible(output: PortValueType, input: PortValueType) -> bool {
    output == input || output == PortValueType::Any || input == PortValueType::Any
}

fn param_value_compatible(kind: ParamKind, value: &serde_json::Value) -> bool {
    if value.is_null() {
        return true;
    }
    match kind {
        ParamKind::String
        | ParamKind::FilePath
        | ParamKind::FolderPath
        | ParamKind::Url
        | ParamKind::Secret
        | ParamKind::Enum => value.is_string(),
        ParamKind::Number => value.is_number(),
        ParamKind::Boolean => value.is_boolean(),
        ParamKind::Json => true,
    }
}

fn edge_id(from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> String {
    format!("{from_node}:{from_port}->{to_node}:{to_port}")
}

fn error(
    code: &'static str,
    message: String,
    node_id: Option<String>,
    edge_id: Option<String>,
    port: Option<String>,
) -> ValidationIssue {
    ValidationIssue {
        level: ValidationLevel::Error,
        code,
        message,
        node_id,
        edge_id,
        port,
    }
}
