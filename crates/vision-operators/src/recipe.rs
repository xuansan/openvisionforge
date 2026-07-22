use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

use crate::schema::schema_with_params;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "recipe.select",
        "Select Recipe",
        "Recipe",
        &[],
        &[],
        &["data"],
        RECIPE_PARAMS,
        select_recipe,
    ));
}

const RECIPE_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "product_code",
        "Product Code",
        ParamKind::String,
        false,
        Some("default"),
        "Product or part number",
        &[],
    )
    .group("Recipe")
    .order(10),
    ParamSchema::new(
        "recipe_version",
        "Recipe Version",
        ParamKind::String,
        false,
        Some("v1"),
        "Recipe version used for traceability",
        &[],
    )
    .group("Recipe")
    .order(20),
    ParamSchema::new(
        "parameters",
        "Parameters",
        ParamKind::Json,
        false,
        Some("{}"),
        "Recipe parameter map such as thresholds and scale",
        &[],
    )
    .group("Recipe")
    .order(30),
];

fn select_recipe(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let product_code = text(&ctx.node.params, "product_code", "default");
    let recipe_version = text(&ctx.node.params, "recipe_version", "v1");
    let parameters = ctx
        .node
        .params
        .get("parameters")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(BTreeMap::from([(
        "data".to_string(),
        Value::Json(serde_json::json!({
            "product_code": product_code,
            "recipe_version": recipe_version,
            "parameters": parameters,
        })),
    )]))
}

fn text(value: &serde_json::Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or(default)
        .to_string()
}
