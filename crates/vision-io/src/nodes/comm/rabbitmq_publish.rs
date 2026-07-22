use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_comm(
        registry,
        "comm.rabbitmq.publish",
        "RabbitMQ Publish",
        rabbitmq_publish,
    );
}

fn rabbitmq_publish(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_message(
        ctx,
        "rabbitmq",
        crate::util::text(&ctx.node.params, "routing_key", "vision.result"),
    )
}
