use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

use crate::schema::{
    enum_param, json_param, number_param, schema_with_params, secret_param, string_param, url_param,
};

pub(crate) fn register_vlm_node(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
    task: &'static str,
) {
    registry.register(schema_with_params(
        id,
        name,
        "Infer/Qwen VLM",
        &[],
        &["image"],
        &["text", "result", "raw"],
        params_for_task(task),
        run_vlm_api,
    ));
}

fn run_vlm_api(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let model = crate::util::text(&ctx.node.params, "model", "qwen3-vl-plus");
    let endpoint = crate::util::text(
        &ctx.node.params,
        "endpoint",
        "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
    );
    let api_key_env = crate::util::text(&ctx.node.params, "api_key_env", "DASHSCOPE_API_KEY");
    let prompt = crate::util::text(
        &ctx.node.params,
        "prompt",
        default_prompt(&ctx.node.operator),
    );
    let image_url = crate::util::text(&ctx.node.params, "image_url", "");
    let max_tokens = crate::util::integer(&ctx.node.params, "max_tokens", 1024);
    let temperature = crate::util::number(&ctx.node.params, "temperature", 0.1);
    let response_format = crate::util::text(&ctx.node.params, "response_format", "json");
    let image_summary = ctx.inputs.get("image").map(Value::summary);
    let text = format!(
        "Qwen VLM API node prepared: model={model}, task={}, image_input={}, image_url={}",
        task_from_operator(&ctx.node.operator),
        image_summary.is_some(),
        !image_url.is_empty()
    );
    let result = serde_json::json!({
        "provider": "qwen",
        "model": model,
        "task": task_from_operator(&ctx.node.operator),
        "endpoint": endpoint,
        "api_key_env": api_key_env,
        "prompt": prompt,
        "image_input": image_summary,
        "image_url": if image_url.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(image_url) },
        "max_tokens": max_tokens,
        "temperature": temperature,
        "response_format": response_format,
        "mode": "prepared",
        "note": "HTTP execution is intentionally not linked yet; this node defines the stable VLM workflow surface."
    });
    Ok(BTreeMap::from([
        ("text".to_string(), Value::Text(text)),
        ("result".to_string(), Value::Json(result.clone())),
        ("raw".to_string(), Value::Json(result)),
    ]))
}

fn task_from_operator(operator: &str) -> &str {
    operator.rsplit('.').next().unwrap_or("describe")
}

fn default_prompt(operator: &str) -> &'static str {
    match task_from_operator(operator) {
        "ocr" => "Read all visible text in the image and return structured OCR results.",
        "grounding" => "Find the requested object or defect in the image and return regions.",
        "inspect" => "Inspect the image for industrial defects and return pass/fail with reasons.",
        _ => "Describe the image and list important visual observations.",
    }
}

const DESCRIBE_PARAMS: &[vision_core::ParamSchema] = &[
    string_param(
        "model",
        "Model",
        false,
        Some("qwen3-vl-plus"),
        "Qwen VLM model name",
    )
    .group("Model")
    .order(10)
    .placeholder("qwen3-vl-plus"),
    url_param(
        "endpoint",
        "Endpoint",
        false,
        Some("https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"),
        "OpenAI-compatible DashScope chat completions endpoint",
    )
    .group("API")
    .order(20)
    .advanced(true),
    secret_param(
        "api_key_env",
        "API Key Env",
        false,
        Some("DASHSCOPE_API_KEY"),
        "Environment variable that stores the API key",
    )
    .group("API")
    .order(30)
    .deployment_required(true)
    .placeholder("DASHSCOPE_API_KEY"),
    string_param("prompt", "Prompt", false, None, "Task prompt")
        .group("Prompt")
        .order(40)
        .placeholder("Inspect this image and return structured result."),
    url_param(
        "image_url",
        "Image URL",
        false,
        None,
        "Optional remote image URL",
    )
    .group("Input")
    .order(50)
    .placeholder("https://example.com/image.jpg"),
    number_param(
        "max_tokens",
        "Max Tokens",
        false,
        Some("1024"),
        "Maximum output tokens",
    )
    .group("Generation")
    .order(60)
    .range(1.0, 8192.0, 1.0),
    number_param(
        "temperature",
        "Temperature",
        false,
        Some("0.1"),
        "Sampling temperature",
    )
    .group("Generation")
    .order(70)
    .range(0.0, 2.0, 0.01),
    enum_param(
        "response_format",
        "Response Format",
        false,
        Some("json"),
        "Preferred response shape",
        &["json", "text"],
    )
    .group("Output")
    .order(80),
    json_param(
        "extra",
        "Extra",
        false,
        Some("{}"),
        "Provider-specific extra parameters",
    )
    .group("Advanced")
    .order(90)
    .advanced(true),
];

const OCR_PARAMS: &[vision_core::ParamSchema] = DESCRIBE_PARAMS;
const GROUNDING_PARAMS: &[vision_core::ParamSchema] = DESCRIBE_PARAMS;
const INSPECT_PARAMS: &[vision_core::ParamSchema] = DESCRIBE_PARAMS;

fn params_for_task(task: &str) -> &'static [vision_core::ParamSchema] {
    match task {
        "ocr" => OCR_PARAMS,
        "grounding" => GROUNDING_PARAMS,
        "inspect" => INSPECT_PARAMS,
        _ => DESCRIBE_PARAMS,
    }
}
