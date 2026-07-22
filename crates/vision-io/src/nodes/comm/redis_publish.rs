use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_comm(
        registry,
        "comm.redis.publish",
        "Redis Publish",
        redis_publish,
    );
}

fn redis_publish(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_message(
        ctx,
        "redis",
        crate::util::text(&ctx.node.params, "channel", "vision:result"),
    )
}
