use std::{error::Error, fmt};

pub type Result<T> = std::result::Result<T, VisionError>;

#[derive(Debug)]
pub enum VisionError {
    Io(String),
    Json(String),
    DuplicateNode(String),
    UnknownOperator(String),
    UnknownNode(String),
    UnknownPort {
        operator: String,
        port: String,
        direction: &'static str,
    },
    MissingInput {
        node: String,
        port: String,
    },
    InputPortAlreadyConnected {
        node: String,
        port: String,
    },
    MissingOutput {
        node: String,
        port: String,
    },
    Cycle,
    Operator {
        node: String,
        message: String,
    },
}

impl fmt::Display for VisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "io error: {msg}"),
            Self::Json(msg) => write!(f, "json error: {msg}"),
            Self::DuplicateNode(node) => write!(f, "duplicate node id: {node}"),
            Self::UnknownOperator(op) => write!(f, "unknown operator: {op}"),
            Self::UnknownNode(node) => write!(f, "unknown node: {node}"),
            Self::UnknownPort {
                operator,
                port,
                direction,
            } => {
                write!(f, "unknown {direction} port {operator}.{port}")
            }
            Self::MissingInput { node, port } => write!(f, "missing input {node}.{port}"),
            Self::InputPortAlreadyConnected { node, port } => {
                write!(f, "input port already connected: {node}.{port}")
            }
            Self::MissingOutput { node, port } => write!(f, "missing output {node}.{port}"),
            Self::Cycle => write!(f, "workflow has a cycle"),
            Self::Operator { node, message } => write!(f, "operator {node} failed: {message}"),
        }
    }
}

impl Error for VisionError {}

impl From<std::io::Error> for VisionError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for VisionError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}
