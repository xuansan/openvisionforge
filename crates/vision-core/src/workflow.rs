use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, serde_json::Value>,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub operator: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub execution: NodeExecutionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeExecutionPolicy {
    #[serde(default)]
    pub run_when: RunCondition,
    #[serde(default)]
    pub always_run: bool,
    #[serde(default)]
    pub short_circuit_on_ng: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunCondition {
    #[default]
    Always,
    AllOk,
    AnyOk,
    AllNg,
    AnyNg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}
