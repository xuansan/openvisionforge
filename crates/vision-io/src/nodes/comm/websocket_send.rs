use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_comm(
        registry,
        "comm.websocket.send",
        "WebSocket Send",
        websocket_send,
    );
}

fn websocket_send(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_message(
        ctx,
        "websocket",
        crate::util::text(&ctx.node.params, "url", "ws://localhost:9000"),
    )
}
