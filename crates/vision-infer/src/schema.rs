use vision_core::{OperatorSchema, ParamKind, ParamSchema};

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

pub(crate) const fn number_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    param(
        key,
        label,
        ParamKind::Number,
        required,
        default,
        description,
        &[],
    )
}

pub(crate) const fn bool_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    param(
        key,
        label,
        ParamKind::Boolean,
        required,
        default,
        description,
        &[],
    )
}

pub(crate) const fn string_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    param(
        key,
        label,
        ParamKind::String,
        required,
        default,
        description,
        &[],
    )
}

pub(crate) const fn url_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    param(
        key,
        label,
        ParamKind::Url,
        required,
        default,
        description,
        &[],
    )
}

pub(crate) const fn secret_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    param(
        key,
        label,
        ParamKind::Secret,
        required,
        default,
        description,
        &[],
    )
}

pub(crate) const fn json_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    param(
        key,
        label,
        ParamKind::Json,
        required,
        default,
        description,
        &[],
    )
}

pub(crate) const fn file_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    param(
        key,
        label,
        ParamKind::FilePath,
        required,
        default,
        description,
        &[],
    )
}

pub(crate) const fn enum_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
    options: &'static [&'static str],
) -> ParamSchema {
    param(
        key,
        label,
        ParamKind::Enum,
        required,
        default,
        description,
        options,
    )
}

const fn param(
    key: &'static str,
    label: &'static str,
    kind: ParamKind,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
    options: &'static [&'static str],
) -> ParamSchema {
    ParamSchema::new(key, label, kind, required, default, description, options)
}
