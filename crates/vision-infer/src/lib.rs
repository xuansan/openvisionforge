mod backend;
mod decode;
mod model_assets;
mod nodes;
#[cfg(feature = "onnxruntime")]
mod preprocess;
mod schema;
mod util;

use vision_core::OperatorRegistry;

pub fn register_all(registry: &mut OperatorRegistry) {
    nodes::onnxruntime_run::register(registry);
    nodes::yolo_detect::register(registry);
    nodes::ultralytics::register(registry);
    nodes::qwen::register(registry);
}
