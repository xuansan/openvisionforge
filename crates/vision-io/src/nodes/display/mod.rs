mod image;
mod json;
mod result;

use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    image::register(registry);
    json::register(registry);
    result::register(registry);
}

fn display_path(ctx: &vision_core::NodeContext<'_>, extension: &str) -> std::path::PathBuf {
    ctx.execution
        .records_dir
        .join("display")
        .join(format!("{}.{}", ctx.node.id, extension))
}
