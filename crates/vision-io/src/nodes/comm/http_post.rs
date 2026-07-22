use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_comm(registry, "comm.http.post", "HTTP Post", http_post);
}

fn http_post(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let url = super::destination_url(ctx, "http://localhost");
    if super::is_real_mode(ctx) {
        return super::send_http_message(ctx, "http", url);
    }

    super::write_message(ctx, "http", url)
}
