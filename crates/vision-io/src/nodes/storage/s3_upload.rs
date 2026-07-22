use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_upload(registry, "storage.s3.upload", "S3 Upload", s3_upload);
}

fn s3_upload(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::upload_manifest(ctx, "s3")
}
