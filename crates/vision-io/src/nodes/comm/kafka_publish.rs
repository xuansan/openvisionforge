use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_comm(
        registry,
        "comm.kafka.publish",
        "Kafka Publish",
        kafka_publish,
    );
}

fn kafka_publish(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_message(
        ctx,
        "kafka",
        crate::util::text(&ctx.node.params, "topic", "vision-result"),
    )
}
