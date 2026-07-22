use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_comm(registry, "comm.mqtt.publish", "MQTT Publish", mqtt_publish);
}

fn mqtt_publish(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_message(
        ctx,
        "mqtt",
        crate::util::text(&ctx.node.params, "topic", "vision/result"),
    )
}
