use vision_core::{OperatorSchema, ParamSchema};

pub(crate) fn schema_with_params(
    id: &'static str,
    name: &'static str,
    category: &'static str,
    inputs: &'static [&'static str],
    optional_inputs: &'static [&'static str],
    outputs: &'static [&'static str],
    params: &'static [ParamSchema],
    run: vision_core::OperatorFn,
) -> OperatorSchema {
    OperatorSchema {
        id,
        name,
        category,
        inputs,
        optional_inputs,
        outputs,
        params,
        run,
    }
}
