use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_comm(registry, "comm.grpc.call", "gRPC Call", grpc_call);
}

fn grpc_call(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_message(
        ctx,
        "grpc",
        crate::util::text(&ctx.node.params, "method", "VisionResultService/Push"),
    )
}
