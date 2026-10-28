use vision_core::{port_schema, OperatorRegistry, OperatorSchema};

pub mod stream;
pub use stream::{BoundedFrameQueue, FolderFrameSource, StreamExecutor};

pub fn registry() -> OperatorRegistry {
    let mut registry = OperatorRegistry::new();
    vision_operators::register_all(&mut registry);
    vision_opencv::register_all(&mut registry);
    vision_io::register_all(&mut registry);
    vision_infer::register_all(&mut registry);
    registry
}

pub fn operator_json(op: &OperatorSchema) -> serde_json::Value {
    serde_json::json!({
        "id": op.id,
        "name": op.name,
        "category": op.category,
        "inputs": op.inputs,
        "optional_inputs": op.optional_inputs,
        "outputs": op.outputs,
        "input_ports": op.inputs.iter().map(|port| port_schema(port, true)).collect::<Vec<_>>(),
        "optional_input_ports": op.optional_inputs.iter().map(|port| port_schema(port, false)).collect::<Vec<_>>(),
        "output_ports": op.outputs.iter().map(|port| port_schema(port, true)).collect::<Vec<_>>(),
        "params": op.params,
    })
}

pub fn export_schema() -> Vec<serde_json::Value> {
    registry().list().map(operator_json).collect()
}
