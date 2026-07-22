use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_comm(registry, "comm.webhook.send", "Webhook Send", webhook_send);
}

fn webhook_send(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let url = super::destination_url(ctx, "http://localhost/webhook");
    if super::is_real_mode(ctx) {
        return super::send_http_message(ctx, "webhook", url);
    }

    super::write_message(ctx, "webhook", url)
}
