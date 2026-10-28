use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

use crate::{
    backend::{bridge_image, integer, one, text},
    schema::schema,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema(
        "opencv.video.read_frame",
        "OpenCV Video Read Frame",
        "OpenCv/Video",
        &[],
        &[],
        &["image"],
        video_read_frame,
    ));
}

fn video_read_frame(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let path = text(&ctx.node.params, "path", "");
    if path.is_empty() {
        return crate::backend::fail("opencv.video.read_frame requires params.path");
    }
    let frame_index = integer(&ctx.node.params, "frame_index", 0);
    let output = ctx
        .execution
        .records_dir
        .join(format!("video_frames/frame_{frame_index:06}.png"));
    let output_text = output.to_string_lossy().to_string();
    let image = bridge_image(&[
        "video_frame",
        path.as_str(),
        &frame_index.to_string(),
        &output_text,
    ])?;
    Ok(one("image", Value::Image(image)))
}
