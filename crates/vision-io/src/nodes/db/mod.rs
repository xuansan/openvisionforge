mod milvus_insert;
mod milvus_search;
mod mongodb_find;
mod mongodb_insert;
mod mysql_insert;
mod mysql_query;
mod pg_insert;
mod pg_query;

use vision_core::{OperatorRegistry, ParamKind, ParamSchema};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    mongodb_insert::register(registry);
    mongodb_find::register(registry);
    mysql_insert::register(registry);
    mysql_query::register(registry);
    pg_insert::register(registry);
    pg_query::register(registry);
    milvus_insert::register(registry);
    milvus_search::register(registry);
}

fn register_db(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
    inputs: &'static [&'static str],
    optional_inputs: &'static [&'static str],
    run: vision_core::OperatorFn,
) {
    registry.register(crate::schema::schema_with_params(
        id,
        name,
        "Database",
        inputs,
        optional_inputs,
        &["operation", "files"],
        params_for(id),
        run,
    ));
}

fn write_operation(
    ctx: &vision_core::NodeContext<'_>,
    database_type: &str,
    operation: &str,
    document: serde_json::Value,
) -> vision_core::Result<std::collections::BTreeMap<String, vision_core::Value>> {
    let message = serde_json::json!({
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "database_type": database_type,
        "operation": operation,
        "uri": crate::util::text(&ctx.node.params, "uri", default_uri(database_type)),
        "database": crate::util::text(&ctx.node.params, "database", "vision"),
        "collection": crate::util::text(&ctx.node.params, "collection", "results"),
        "table": crate::util::text(&ctx.node.params, "table", "inspection_results"),
        "document": document,
        "inputs": crate::util::input_json(ctx),
        "mode": "outbox",
    });
    let output = crate::util::output_path(ctx, "outbox", "db", "json");
    let path = crate::util::write_json(output, &message)?;
    Ok(std::collections::BTreeMap::from([
        ("operation".to_string(), vision_core::Value::Json(message)),
        ("files".to_string(), crate::util::file_value(path)),
    ]))
}

fn default_uri(database_type: &str) -> &'static str {
    match database_type {
        "mysql" => "mysql://localhost:3306",
        "pg" => "postgres://localhost:5432",
        "milvus" => "http://localhost:19530",
        _ => "mongodb://localhost:27017",
    }
}

fn sql_document(ctx: &vision_core::NodeContext<'_>) -> serde_json::Value {
    serde_json::json!({
        "sql": crate::util::text(&ctx.node.params, "sql", ""),
        "params": ctx.node.params.get("params").cloned().unwrap_or_else(|| serde_json::json!([])),
    })
}

fn vector_document(ctx: &vision_core::NodeContext<'_>) -> serde_json::Value {
    serde_json::json!({
        "collection": crate::util::text(&ctx.node.params, "collection", "inspection_vectors"),
        "vector": ctx.node.params.get("vector").cloned().unwrap_or_else(|| serde_json::json!([])),
        "top_k": ctx.node.params.get("top_k").cloned().unwrap_or_else(|| serde_json::json!(10)),
        "filter": ctx.node.params.get("filter").cloned().unwrap_or_else(|| serde_json::json!({})),
        "payload": crate::util::payload(ctx),
    })
}

fn params_for(id: &str) -> &'static [ParamSchema] {
    match id {
        "db.mongodb.insert" => MONGODB_INSERT_PARAMS,
        "db.mongodb.find" => MONGODB_FIND_PARAMS,
        "db.mysql.insert" => MYSQL_INSERT_PARAMS,
        "db.mysql.query" => MYSQL_QUERY_PARAMS,
        "db.pg.insert" => PG_INSERT_PARAMS,
        "db.pg.query" => PG_QUERY_PARAMS,
        "db.milvus.insert" => MILVUS_INSERT_PARAMS,
        "db.milvus.search" => MILVUS_SEARCH_PARAMS,
        _ => &[],
    }
}

const MONGODB_INSERT_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("mongodb://localhost:27017"),
        "MongoDB connection URI",
    ),
    string_param(
        "database",
        "Database",
        true,
        Some("vision"),
        "Database name",
    ),
    string_param(
        "collection",
        "Collection",
        true,
        Some("results"),
        "Collection name",
    ),
    file_param(
        "outbox",
        "Outbox",
        false,
        None,
        "Outbox JSON path for MVP mode",
    ),
];

const MONGODB_FIND_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("mongodb://localhost:27017"),
        "MongoDB connection URI",
    ),
    string_param(
        "database",
        "Database",
        true,
        Some("vision"),
        "Database name",
    ),
    string_param(
        "collection",
        "Collection",
        true,
        Some("results"),
        "Collection name",
    ),
    json_param(
        "filter",
        "Filter",
        false,
        Some("{}"),
        "MongoDB filter document",
    ),
    file_param(
        "outbox",
        "Outbox",
        false,
        None,
        "Outbox JSON path for MVP mode",
    ),
];

const MYSQL_INSERT_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("mysql://localhost:3306"),
        "MySQL connection URI",
    ),
    string_param(
        "database",
        "Database",
        true,
        Some("vision"),
        "Database name",
    ),
    string_param(
        "table",
        "Table",
        true,
        Some("inspection_results"),
        "Table name",
    ),
    file_param(
        "outbox",
        "Outbox",
        false,
        None,
        "Outbox JSON path for MVP mode",
    ),
];

const MYSQL_QUERY_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("mysql://localhost:3306"),
        "MySQL connection URI",
    ),
    string_param(
        "database",
        "Database",
        true,
        Some("vision"),
        "Database name",
    ),
    string_param("sql", "SQL", true, None, "SQL query text"),
    json_param("params", "Params", false, Some("[]"), "SQL bind parameters"),
    file_param(
        "outbox",
        "Outbox",
        false,
        None,
        "Outbox JSON path for MVP mode",
    ),
];

const PG_INSERT_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("postgres://localhost:5432"),
        "PostgreSQL connection URI",
    ),
    string_param(
        "database",
        "Database",
        true,
        Some("vision"),
        "Database name",
    ),
    string_param(
        "table",
        "Table",
        true,
        Some("inspection_results"),
        "Table name",
    ),
    file_param(
        "outbox",
        "Outbox",
        false,
        None,
        "Outbox JSON path for MVP mode",
    ),
];

const PG_QUERY_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("postgres://localhost:5432"),
        "PostgreSQL connection URI",
    ),
    string_param(
        "database",
        "Database",
        true,
        Some("vision"),
        "Database name",
    ),
    string_param("sql", "SQL", true, None, "SQL query text"),
    json_param("params", "Params", false, Some("[]"), "SQL bind parameters"),
    file_param(
        "outbox",
        "Outbox",
        false,
        None,
        "Outbox JSON path for MVP mode",
    ),
];

const MILVUS_INSERT_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("http://localhost:19530"),
        "Milvus endpoint",
    ),
    string_param(
        "database",
        "Database",
        false,
        Some("vision"),
        "Database name",
    ),
    string_param(
        "collection",
        "Collection",
        true,
        Some("inspection_vectors"),
        "Collection name",
    ),
    json_param(
        "vector",
        "Vector",
        false,
        Some("[]"),
        "Vector value or mapping",
    ),
    json_param(
        "filter",
        "Filter",
        false,
        Some("{}"),
        "Optional filter expression payload",
    ),
    file_param(
        "outbox",
        "Outbox",
        false,
        None,
        "Outbox JSON path for MVP mode",
    ),
];

const MILVUS_SEARCH_PARAMS: &[ParamSchema] = &[
    string_param(
        "uri",
        "URI",
        true,
        Some("http://localhost:19530"),
        "Milvus endpoint",
    ),
    string_param(
        "database",
        "Database",
        false,
        Some("vision"),
        "Database name",
    ),
    string_param(
        "collection",
        "Collection",
        true,
        Some("inspection_vectors"),
        "Collection name",
    ),
    json_param(
        "vector",
        "Vector",
        true,
        Some("[]"),
        "Query vector value or mapping",
    ),
    number_param(
        "top_k",
        "Top K",
        false,
        Some("10"),
        "Number of nearest results",
    ),
    json_param(
        "filter",
        "Filter",
        false,
        Some("{}"),
        "Optional scalar filter",
    ),
    file_param(
        "outbox",
        "Outbox",
        false,
        None,
        "Outbox JSON path for MVP mode",
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
    )
}

const fn json_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    param(key, label, ParamKind::Json, required, default, description)
}

const fn file_param(
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
    )
}

const fn param(
    key: &'static str,
    label: &'static str,
    kind: ParamKind,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    ParamSchema::new(key, label, kind, required, default, description, &[]).group("Database")
}
