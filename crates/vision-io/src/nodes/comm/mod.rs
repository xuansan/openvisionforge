mod grpc_call;
mod http_post;
mod kafka_publish;
mod mqtt_publish;
mod rabbitmq_publish;
mod redis_publish;
mod webhook_send;
mod websocket_send;

use vision_core::{OperatorRegistry, ParamKind, ParamSchema};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    websocket_send::register(registry);
    http_post::register(registry);
    webhook_send::register(registry);
    mqtt_publish::register(registry);
    rabbitmq_publish::register(registry);
    kafka_publish::register(registry);
    redis_publish::register(registry);
    grpc_call::register(registry);
}

fn register_comm(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
    run: vision_core::OperatorFn,
) {
    registry.register(crate::schema::schema_with_params(
        id,
        name,
        "Communication",
        &[],
        &["payload", "data", "stats", "status", "signal"],
        &["message", "files"],
        params_for(id),
        run,
    ));
}

fn write_message(
    ctx: &vision_core::NodeContext<'_>,
    transport: &str,
    destination: String,
) -> vision_core::Result<std::collections::BTreeMap<String, vision_core::Value>> {
    let message = serde_json::json!({
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "transport": transport,
        "destination": destination,
        "params": ctx.node.params,
        "payload": crate::util::payload(ctx),
        "inputs": crate::util::input_json(ctx),
        "mode": "outbox",
    });
    let output = crate::util::output_path(ctx, "outbox", "comm", "json");
    let path = crate::util::write_json(output, &message)?;
    Ok(std::collections::BTreeMap::from([
        ("message".to_string(), vision_core::Value::Json(message)),
        ("files".to_string(), crate::util::file_value(path)),
    ]))
}

fn send_http_message(
    ctx: &vision_core::NodeContext<'_>,
    transport: &str,
    destination: String,
) -> vision_core::Result<std::collections::BTreeMap<String, vision_core::Value>> {
    let body = request_body(ctx);
    let body_text = serde_json::to_string(&body)?;
    let mut output = std::process::Command::new("curl")
        .args([
            "-sS",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            &body_text,
            &destination,
        ])
        .output()
        .map_err(|err| vision_core::VisionError::Operator {
            node: ctx.node.id.clone(),
            message: format!("failed to send {transport} request: {err}"),
        })?;

    if !output.status.success() {
        output = python_http_post(&destination, &body_text).map_err(|err| {
            vision_core::VisionError::Operator {
                node: ctx.node.id.clone(),
                message: format!("failed to send {transport} request with python fallback: {err}"),
            }
        })?;
    }

    let status = output.status.code().unwrap_or(-1);
    let response_body = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let message = serde_json::json!({
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "transport": transport,
        "destination": redact_destination(&destination),
        "payload": body,
        "status": status,
        "response": response_body,
        "stderr": stderr,
        "mode": "real",
    });
    let output = crate::util::output_path(ctx, "outbox", "comm", "json");
    let path = crate::util::write_json(output, &message)?;
    Ok(std::collections::BTreeMap::from([
        ("message".to_string(), vision_core::Value::Json(message)),
        ("files".to_string(), crate::util::file_value(path)),
    ]))
}

fn python_http_post(destination: &str, body_text: &str) -> std::io::Result<std::process::Output> {
    let mut child = std::process::Command::new("python")
        .arg("-c")
        .arg(
            "import sys, urllib.request; \
             url=sys.argv[1]; \
             body=sys.stdin.buffer.read(); \
             req=urllib.request.Request(url,data=body,headers={'Content-Type':'application/json'},method='POST'); \
             resp=urllib.request.urlopen(req,timeout=20); \
             print(resp.read().decode('utf-8'))",
        )
        .arg(destination)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(body_text.as_bytes())?;
    }
    child.wait_with_output()
}

fn redact_destination(destination: &str) -> String {
    if let Some((base, _)) = destination.split_once("key=") {
        return format!("{base}key=<redacted>");
    }
    destination.to_string()
}

fn request_body(ctx: &vision_core::NodeContext<'_>) -> serde_json::Value {
    if let Some(body) = ctx.node.params.get("body") {
        return body.clone();
    }

    let payload = crate::util::payload(ctx);
    let wechat_text = ctx
        .node
        .params
        .get("wechat_text")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !wechat_text {
        return payload;
    }

    let prefix = crate::util::text(&ctx.node.params, "text_prefix", "vision-platform result");
    let payload_text =
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    serde_json::json!({
        "msgtype": "text",
        "text": {
            "content": format!("{prefix}\n{payload_text}")
        }
    })
}

fn destination_url(ctx: &vision_core::NodeContext<'_>, default: &str) -> String {
    if let Some(env_name) = ctx.node.params.get("url_env").and_then(|v| v.as_str()) {
        if let Ok(value) = std::env::var(env_name) {
            if !value.is_empty() {
                return value;
            }
        }
    }

    crate::util::text(&ctx.node.params, "url", default)
}

fn is_real_mode(ctx: &vision_core::NodeContext<'_>) -> bool {
    ctx.node
        .params
        .get("mode")
        .and_then(|v| v.as_str())
        .is_some_and(|mode| mode == "real")
}

fn params_for(id: &str) -> &'static [ParamSchema] {
    match id {
        "comm.kafka.publish" => KAFKA_PARAMS,
        "comm.rabbitmq.publish" => RABBITMQ_PARAMS,
        "comm.mqtt.publish" => MQTT_PARAMS,
        "comm.redis.publish" => REDIS_PARAMS,
        "comm.websocket.send" => WEBSOCKET_PARAMS,
        "comm.http.post" => HTTP_PARAMS,
        "comm.webhook.send" => WEBHOOK_PARAMS,
        "comm.grpc.call" => GRPC_PARAMS,
        _ => &[],
    }
}

const KAFKA_PARAMS: &[ParamSchema] = &[
    string_param(
        "brokers",
        "Brokers",
        true,
        Some("localhost:9092"),
        "Kafka broker list",
    ),
    string_param("topic", "Topic", true, Some("vision-result"), "Kafka topic"),
    string_param("key", "Key", false, None, "Optional message key"),
    json_param("headers", "Headers", false, Some("{}"), "Message headers"),
    number_param(
        "partition",
        "Partition",
        false,
        None,
        "Optional target partition",
    ),
    enum_param(
        "acks",
        "Acks",
        false,
        Some("all"),
        "Acknowledgement mode",
        &["0", "1", "all"],
    ),
    number_param(
        "timeout_ms",
        "Timeout Ms",
        false,
        Some("3000"),
        "Send timeout in milliseconds",
    ),
    enum_param(
        "compression",
        "Compression",
        false,
        Some("none"),
        "Compression codec",
        &["none", "gzip", "snappy", "lz4", "zstd"],
    ),
    enum_param(
        "payload_format",
        "Payload Format",
        false,
        Some("json"),
        "Payload serialization format",
        &["json", "text", "bytes"],
    ),
];

const RABBITMQ_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("amqp://guest:guest@localhost:5672/%2f"),
        "RabbitMQ connection URI",
    ),
    string_param("exchange", "Exchange", false, Some(""), "Exchange name"),
    string_param(
        "routing_key",
        "Routing Key",
        true,
        Some("vision.result"),
        "Routing key",
    ),
    string_param("queue", "Queue", false, None, "Optional target queue"),
    bool_param(
        "durable",
        "Durable",
        false,
        Some("true"),
        "Use durable routing resources",
    ),
    json_param("headers", "Headers", false, Some("{}"), "Message headers"),
    string_param(
        "content_type",
        "Content Type",
        false,
        Some("application/json"),
        "Message content type",
    ),
    enum_param(
        "delivery_mode",
        "Delivery Mode",
        false,
        Some("persistent"),
        "Delivery mode",
        &["transient", "persistent"],
    ),
    number_param(
        "timeout_ms",
        "Timeout Ms",
        false,
        Some("3000"),
        "Send timeout in milliseconds",
    ),
];

const MQTT_PARAMS: &[ParamSchema] = &[
    string_param(
        "broker",
        "Broker",
        true,
        Some("localhost"),
        "MQTT broker host",
    ),
    number_param("port", "Port", true, Some("1883"), "MQTT broker port"),
    string_param(
        "client_id",
        "Client ID",
        false,
        Some("vision-platform"),
        "Client identifier",
    ),
    string_param("username", "Username", false, None, "Username"),
    secret_param("password", "Password", false, None, "Password"),
    string_param("topic", "Topic", true, Some("vision/result"), "MQTT topic"),
    enum_param(
        "qos",
        "QoS",
        false,
        Some("1"),
        "Quality of service",
        &["0", "1", "2"],
    ),
    bool_param("retain", "Retain", false, Some("false"), "Retain message"),
    bool_param(
        "clean_session",
        "Clean Session",
        false,
        Some("true"),
        "Clean session flag",
    ),
    number_param(
        "timeout_ms",
        "Timeout Ms",
        false,
        Some("3000"),
        "Send timeout in milliseconds",
    ),
];

const REDIS_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("redis://localhost:6379"),
        "Redis connection URI",
    ),
    string_param(
        "channel",
        "Channel",
        true,
        Some("vision:result"),
        "Pub/Sub channel",
    ),
    number_param(
        "database",
        "Database",
        false,
        Some("0"),
        "Redis database number",
    ),
    string_param("username", "Username", false, None, "Username"),
    secret_param("password", "Password", false, None, "Password"),
    number_param(
        "timeout_ms",
        "Timeout Ms",
        false,
        Some("3000"),
        "Send timeout in milliseconds",
    ),
];

const WEBSOCKET_PARAMS: &[ParamSchema] = &[
    url_param(
        "url",
        "URL",
        true,
        Some("ws://localhost:9000"),
        "WebSocket URL",
    ),
    json_param(
        "headers",
        "Headers",
        false,
        Some("{}"),
        "Connection headers",
    ),
    bool_param(
        "reconnect",
        "Reconnect",
        false,
        Some("true"),
        "Reconnect on failure",
    ),
    number_param(
        "heartbeat_ms",
        "Heartbeat Ms",
        false,
        Some("30000"),
        "Heartbeat interval",
    ),
    number_param(
        "timeout_ms",
        "Timeout Ms",
        false,
        Some("3000"),
        "Send timeout in milliseconds",
    ),
];

const HTTP_PARAMS: &[ParamSchema] = &[
    url_param(
        "url",
        "URL",
        true,
        Some("http://localhost"),
        "HTTP endpoint",
    ),
    enum_param(
        "method",
        "Method",
        false,
        Some("POST"),
        "HTTP method",
        &["POST", "PUT", "PATCH"],
    ),
    json_param("headers", "Headers", false, Some("{}"), "Request headers"),
    json_param("query", "Query", false, Some("{}"), "Query parameters"),
    enum_param(
        "auth_type",
        "Auth Type",
        false,
        Some("none"),
        "Authentication type",
        &["none", "bearer", "basic", "api_key"],
    ),
    number_param(
        "timeout_ms",
        "Timeout Ms",
        false,
        Some("3000"),
        "Request timeout in milliseconds",
    ),
    number_param("retry", "Retry", false, Some("0"), "Retry count"),
    enum_param(
        "payload_format",
        "Payload Format",
        false,
        Some("json"),
        "Payload serialization format",
        &["json", "text", "bytes"],
    ),
    enum_param(
        "mode",
        "Mode",
        false,
        Some("outbox"),
        "Send mode",
        &["outbox", "real"],
    ),
    json_param("body", "Body", false, None, "Override request JSON body"),
    bool_param(
        "wechat_text",
        "WeChat Text",
        false,
        Some("false"),
        "Wrap payload as Enterprise WeChat text message",
    ),
    string_param(
        "text_prefix",
        "Text Prefix",
        false,
        Some("vision-platform result"),
        "Text prefix for wechat_text mode",
    ),
    string_param(
        "url_env",
        "URL Env",
        false,
        None,
        "Environment variable containing the URL",
    ),
];

const WEBHOOK_PARAMS: &[ParamSchema] = &[
    url_param(
        "url",
        "URL",
        true,
        Some("http://localhost/webhook"),
        "Webhook endpoint",
    ),
    json_param("headers", "Headers", false, Some("{}"), "Request headers"),
    enum_param(
        "auth_type",
        "Auth Type",
        false,
        Some("none"),
        "Authentication type",
        &["none", "bearer", "basic", "api_key"],
    ),
    number_param(
        "timeout_ms",
        "Timeout Ms",
        false,
        Some("3000"),
        "Request timeout in milliseconds",
    ),
    number_param("retry", "Retry", false, Some("0"), "Retry count"),
    enum_param(
        "payload_format",
        "Payload Format",
        false,
        Some("json"),
        "Payload serialization format",
        &["json", "text", "bytes"],
    ),
    enum_param(
        "mode",
        "Mode",
        false,
        Some("outbox"),
        "Send mode",
        &["outbox", "real"],
    ),
    json_param("body", "Body", false, None, "Override request JSON body"),
    string_param(
        "url_env",
        "URL Env",
        false,
        None,
        "Environment variable containing the URL",
    ),
];

const GRPC_PARAMS: &[ParamSchema] = &[
    url_param(
        "endpoint",
        "Endpoint",
        true,
        Some("http://localhost:50051"),
        "gRPC endpoint",
    ),
    string_param(
        "service",
        "Service",
        true,
        Some("VisionResultService"),
        "Service name",
    ),
    string_param(
        "method",
        "Method",
        true,
        Some("VisionResultService/Push"),
        "Method name",
    ),
    json_param(
        "metadata",
        "Metadata",
        false,
        Some("{}"),
        "Request metadata",
    ),
    bool_param("tls", "TLS", false, Some("false"), "Enable TLS"),
    number_param(
        "timeout_ms",
        "Timeout Ms",
        false,
        Some("3000"),
        "Request timeout in milliseconds",
    ),
    json_param(
        "request_schema",
        "Request Schema",
        false,
        Some("{}"),
        "Request schema mapping",
    ),
];

const fn string_param(
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

const fn number_param(
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

const fn bool_param(
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

const fn json_param(
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

const fn url_param(
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

const fn secret_param(
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

const fn enum_param(
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
        .group("Communication")
}
