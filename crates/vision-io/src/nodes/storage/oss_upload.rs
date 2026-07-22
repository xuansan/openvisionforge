use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_upload(registry, "storage.oss.upload", "OSS Upload", oss_upload);
}

fn oss_upload(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::upload_manifest(ctx, "oss")
}
