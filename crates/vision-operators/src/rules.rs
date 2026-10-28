use std::collections::BTreeMap;

use vision_core::{
    FinalResult, NodeContext, OperatorRegistry, ParamKind, ParamSchema, QualityLevel, Result,
    RuleResult, Value,
};

use crate::schema::schema_with_params;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "rule.numeric_range",
        "Numeric Range Rule",
        "Rules/Judge",
        &["data"],
        &[],
        &["rule", "passed"],
        NUMERIC_RANGE_PARAMS,
        numeric_range,
    ));
    registry.register(schema_with_params(
        "rule.count_range",
        "Count Range Rule",
        "Rules/Judge",
        &["data"],
        &[],
        &["rule", "passed"],
        COUNT_RANGE_PARAMS,
        count_range,
    ));
    registry.register(schema_with_params(
        "rule.bool",
        "Boolean Rule",
        "Rules/Judge",
        &["data"],
        &[],
        &["rule", "passed"],
        BOOL_PARAMS,
        bool_rule,
    ));
    registry.register(schema_with_params(
        "rule.string_match",
        "String Match Rule",
        "Rules/Judge",
        &["data"],
        &[],
        &["rule", "passed"],
        STRING_PARAMS,
        string_match,
    ));
    registry.register(schema_with_params(
        "rule.confidence_min",
        "Confidence Min Rule",
        "Rules/Judge",
        &["data"],
        &[],
        &["rule", "passed"],
        CONFIDENCE_PARAMS,
        confidence_min,
    ));
    registry.register(schema_with_params(
        "rule.class_allowed",
        "Class Allowed Rule",
        "Rules/Judge",
        &["data"],
        &[],
        &["rule", "passed"],
        CLASS_PARAMS,
        class_allowed,
    ));
    registry.register(schema_with_params(
        "rule.region_contains",
        "Region Contains Rule",
        "Rules/Judge",
        &["data"],
        &[],
        &["rule", "passed"],
        REGION_PARAMS,
        region_contains,
    ));
    registry.register(schema_with_params(
        "logic.and",
        "AND",
        "Rules/Logic",
        &["rule"],
        &["rule2", "rule3", "rule4", "rule5", "rules"],
        &["rule", "passed"],
        LOGIC_PARAMS,
        logic_and,
    ));
    registry.register(schema_with_params(
        "logic.all",
        "All Rules",
        "Rules/Logic",
        &["rule"],
        &["rule2", "rule3", "rule4", "rule5", "rules"],
        &["rule", "passed"],
        LOGIC_PARAMS,
        logic_and,
    ));
    registry.register(schema_with_params(
        "logic.or",
        "OR",
        "Rules/Logic",
        &["rule"],
        &["rule2", "rule3", "rule4", "rule5", "rules"],
        &["rule", "passed"],
        LOGIC_PARAMS,
        logic_or,
    ));
    registry.register(schema_with_params(
        "logic.any",
        "Any Rule",
        "Rules/Logic",
        &["rule"],
        &["rule2", "rule3", "rule4", "rule5", "rules"],
        &["rule", "passed"],
        LOGIC_PARAMS,
        logic_or,
    ));
    registry.register(schema_with_params(
        "logic.not",
        "NOT",
        "Rules/Logic",
        &["rule"],
        &[],
        &["rule", "passed"],
        LOGIC_PARAMS,
        logic_not,
    ));
    registry.register(schema_with_params(
        "logic.n_of_m",
        "N of M Rules",
        "Rules/Logic",
        &["rule"],
        &["rule2", "rule3", "rule4", "rule5", "rules"],
        &["rule", "passed"],
        N_OF_M_PARAMS,
        logic_n_of_m,
    ));
    registry.register(schema_with_params(
        "logic.final_result",
        "Final Result",
        "Rules/Result",
        &["rule"],
        &["rule2", "rule3", "rule4", "rule5", "rules"],
        &["final", "status", "passed"],
        FINAL_PARAMS,
        final_result,
    ));
}

const NUMERIC_RANGE_PARAMS: &[ParamSchema] = &[
    base_rule_id(),
    ParamSchema::new(
        "field",
        "Field",
        ParamKind::String,
        false,
        Some(""),
        "Field path to read",
        &[],
    )
    .group("Rule")
    .order(20)
    .placeholder("area_px"),
    ParamSchema::new(
        "min",
        "Min",
        ParamKind::Number,
        false,
        None,
        "Minimum inclusive value",
        &[],
    )
    .group("Threshold")
    .order(30),
    ParamSchema::new(
        "max",
        "Max",
        ParamKind::Number,
        false,
        None,
        "Maximum inclusive value",
        &[],
    )
    .group("Threshold")
    .order(40),
];

const COUNT_RANGE_PARAMS: &[ParamSchema] = &[
    base_rule_id(),
    ParamSchema::new(
        "min",
        "Min Count",
        ParamKind::Number,
        false,
        Some("0"),
        "Minimum count",
        &[],
    )
    .group("Threshold")
    .order(20)
    .range(0.0, 1000000.0, 1.0),
    ParamSchema::new(
        "max",
        "Max Count",
        ParamKind::Number,
        false,
        Some("999999"),
        "Maximum count",
        &[],
    )
    .group("Threshold")
    .order(30)
    .range(0.0, 1000000.0, 1.0),
];

const BOOL_PARAMS: &[ParamSchema] = &[
    base_rule_id(),
    ParamSchema::new(
        "field",
        "Field",
        ParamKind::String,
        false,
        Some(""),
        "Optional bool field path",
        &[],
    )
    .group("Rule")
    .order(20),
    ParamSchema::new(
        "expected",
        "Expected",
        ParamKind::Boolean,
        false,
        Some("true"),
        "Expected bool",
        &[],
    )
    .group("Rule")
    .order(30),
];

const STRING_PARAMS: &[ParamSchema] = &[
    base_rule_id(),
    ParamSchema::new(
        "field",
        "Field",
        ParamKind::String,
        false,
        Some(""),
        "String field path",
        &[],
    )
    .group("Rule")
    .order(20),
    ParamSchema::new(
        "mode",
        "Mode",
        ParamKind::Enum,
        false,
        Some("equals"),
        "String compare mode",
        &["equals", "contains"],
    )
    .group("Rule")
    .order(30),
    ParamSchema::new(
        "expected",
        "Expected",
        ParamKind::String,
        true,
        None,
        "Expected text",
        &[],
    )
    .group("Rule")
    .order(40),
];

const CONFIDENCE_PARAMS: &[ParamSchema] = &[
    base_rule_id(),
    ParamSchema::new(
        "field",
        "Field",
        ParamKind::String,
        false,
        Some("confidence"),
        "Confidence field path",
        &[],
    )
    .group("Rule")
    .order(20),
    ParamSchema::new(
        "min",
        "Min Confidence",
        ParamKind::Number,
        false,
        Some("0.5"),
        "Minimum confidence",
        &[],
    )
    .group("Threshold")
    .order(30)
    .range(0.0, 1.0, 0.01),
];

const CLASS_PARAMS: &[ParamSchema] = &[
    base_rule_id(),
    ParamSchema::new(
        "allowed",
        "Allowed Classes",
        ParamKind::String,
        true,
        None,
        "Comma separated class names",
        &[],
    )
    .group("Rule")
    .order(20)
    .placeholder("ok,part"),
];

const REGION_PARAMS: &[ParamSchema] = &[
    base_rule_id(),
    ParamSchema::new(
        "x",
        "ROI X",
        ParamKind::Number,
        false,
        Some("0"),
        "ROI left",
        &[],
    )
    .group("ROI")
    .order(20),
    ParamSchema::new(
        "y",
        "ROI Y",
        ParamKind::Number,
        false,
        Some("0"),
        "ROI top",
        &[],
    )
    .group("ROI")
    .order(30),
    ParamSchema::new(
        "width",
        "ROI Width",
        ParamKind::Number,
        false,
        Some("999999"),
        "ROI width",
        &[],
    )
    .group("ROI")
    .order(40),
    ParamSchema::new(
        "height",
        "ROI Height",
        ParamKind::Number,
        false,
        Some("999999"),
        "ROI height",
        &[],
    )
    .group("ROI")
    .order(50),
];

const LOGIC_PARAMS: &[ParamSchema] = &[base_rule_id()];
const N_OF_M_PARAMS: &[ParamSchema] = &[
    base_rule_id(),
    ParamSchema::new(
        "required_passes",
        "Required Passes",
        ParamKind::Number,
        true,
        Some("1"),
        "Minimum number of input rules that must pass",
        &[],
    )
    .group("Logic")
    .order(20)
    .range(1.0, 1000.0, 1.0),
];
const FINAL_PARAMS: &[ParamSchema] = &[base_rule_id()];

const fn base_rule_id() -> ParamSchema {
    ParamSchema::new(
        "rule_id",
        "Rule ID",
        ParamKind::String,
        false,
        Some("rule"),
        "Stable rule identifier used in traces and final result",
        &[],
    )
    .group("Rule")
    .order(10)
}

fn numeric_range(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let field = text(&ctx.node.params, "field", "");
    let actual = value_number(ctx.inputs.get("data"), &field);
    let min = optional_number(&ctx.node.params, "min");
    let max = optional_number(&ctx.node.params, "max");
    let ok = actual
        .map(|value| min.is_none_or(|min| value >= min) && max.is_none_or(|max| value <= max))
        .unwrap_or(false);
    let rule = rule(
        ctx,
        ok,
        actual.map_or(serde_json::Value::Null, serde_json::Value::from),
        serde_json::json!({ "min": min, "max": max }),
        if ok {
            "数值在范围内"
        } else {
            "数值超出范围或不存在"
        },
        Some(field),
    );
    rule_out(rule)
}

fn count_range(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let count = value_count(ctx.inputs.get("data"));
    let min = number(&ctx.node.params, "min", 0.0) as usize;
    let max = number(&ctx.node.params, "max", 999999.0) as usize;
    let ok = count >= min && count <= max;
    rule_out(rule(
        ctx,
        ok,
        serde_json::Value::from(count),
        serde_json::json!({ "min": min, "max": max }),
        if ok {
            "数量在范围内"
        } else {
            "数量超出范围"
        },
        None,
    ))
}

fn bool_rule(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let expected = bool_param(&ctx.node.params, "expected", true);
    let field = text(&ctx.node.params, "field", "");
    let actual = value_bool(ctx.inputs.get("data"), &field);
    let ok = actual == Some(expected);
    rule_out(rule(
        ctx,
        ok,
        actual.map_or(serde_json::Value::Null, serde_json::Value::from),
        serde_json::Value::from(expected),
        if ok {
            "布尔结果符合预期"
        } else {
            "布尔结果不符合预期"
        },
        Some(field),
    ))
}

fn string_match(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let field = text(&ctx.node.params, "field", "");
    let actual = value_text(ctx.inputs.get("data"), &field).unwrap_or_default();
    let expected = text(&ctx.node.params, "expected", "");
    let mode = text(&ctx.node.params, "mode", "equals");
    let ok = match mode.as_str() {
        "contains" => actual.contains(&expected),
        _ => actual == expected,
    };
    rule_out(rule(
        ctx,
        ok,
        serde_json::Value::from(actual),
        serde_json::json!({ "mode": mode, "expected": expected }),
        if ok {
            "字符串符合规则"
        } else {
            "字符串不符合规则"
        },
        Some(field),
    ))
}

fn confidence_min(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let field = text(&ctx.node.params, "field", "confidence");
    let actual = value_number(ctx.inputs.get("data"), &field);
    let min = number(&ctx.node.params, "min", 0.5);
    let ok = actual.is_some_and(|value| value >= min);
    rule_out(rule(
        ctx,
        ok,
        actual.map_or(serde_json::Value::Null, serde_json::Value::from),
        serde_json::json!({ "min": min }),
        if ok {
            "置信度达标"
        } else {
            "置信度不足"
        },
        Some(field),
    ))
}

fn class_allowed(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let allowed = text(&ctx.node.params, "allowed", "")
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let classes = classes(ctx.inputs.get("data"));
    let ok = !classes.is_empty()
        && classes
            .iter()
            .all(|class_name| allowed.iter().any(|allowed| allowed == class_name));
    rule_out(rule(
        ctx,
        ok,
        serde_json::Value::from(classes.join(",")),
        serde_json::Value::from(allowed.join(",")),
        if ok {
            "类别在允许列表内"
        } else {
            "存在不允许类别或没有类别"
        },
        Some("class".to_string()),
    ))
}

fn region_contains(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let x = number(&ctx.node.params, "x", 0.0);
    let y = number(&ctx.node.params, "y", 0.0);
    let width = number(&ctx.node.params, "width", 999999.0);
    let height = number(&ctx.node.params, "height", 999999.0);
    let ok = boxes(ctx.inputs.get("data")).into_iter().all(|bbox| {
        bbox.0 >= x && bbox.1 >= y && bbox.0 + bbox.2 <= x + width && bbox.1 + bbox.3 <= y + height
    });
    rule_out(rule(
        ctx,
        ok,
        serde_json::json!({ "checked": true }),
        serde_json::json!({ "x": x, "y": y, "width": width, "height": height }),
        if ok {
            "区域关系满足"
        } else {
            "目标超出允许区域"
        },
        Some("bbox".to_string()),
    ))
}

fn logic_and(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let rules = collect_rules(ctx);
    let ok = !rules.is_empty() && rules.iter().all(|rule| rule.ok);
    rule_out(rule(
        ctx,
        ok,
        serde_json::json!(rules.iter().map(|rule| rule.ok).collect::<Vec<_>>()),
        serde_json::Value::from("all true"),
        if ok {
            "全部规则通过"
        } else {
            "存在规则未通过"
        },
        None,
    ))
}

fn logic_or(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let rules = collect_rules(ctx);
    let ok = rules.iter().any(|rule| rule.ok);
    rule_out(rule(
        ctx,
        ok,
        serde_json::json!(rules.iter().map(|rule| rule.ok).collect::<Vec<_>>()),
        serde_json::Value::from("any true"),
        if ok {
            "至少一个规则通过"
        } else {
            "没有规则通过"
        },
        None,
    ))
}

fn logic_not(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let input = collect_rules(ctx).into_iter().next();
    let ok = input.as_ref().is_some_and(|rule| !rule.ok);
    rule_out(rule(
        ctx,
        ok,
        input.map_or(serde_json::Value::Null, |rule| {
            serde_json::Value::from(rule.ok)
        }),
        serde_json::Value::from("not true"),
        if ok {
            "NOT 规则通过"
        } else {
            "NOT 规则未通过"
        },
        None,
    ))
}

fn logic_n_of_m(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let rules = collect_rules(ctx);
    let required = number(&ctx.node.params, "required_passes", 1.0) as usize;
    let passed = rules.iter().filter(|rule| rule.ok).count();
    let ok = !rules.is_empty() && required > 0 && passed >= required;
    rule_out(rule(
        ctx,
        ok,
        serde_json::json!({ "passed": passed, "total": rules.len() }),
        serde_json::json!({ "required_passes": required }),
        if ok {
            "通过规则数量达到要求"
        } else {
            "通过规则数量不足"
        },
        None,
    ))
}

fn final_result(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let rules = collect_rules(ctx);
    let failed_rules = rules
        .iter()
        .filter(|rule| !rule.ok)
        .cloned()
        .collect::<Vec<_>>();
    let final_ok = failed_rules.is_empty() && !rules.is_empty();
    let status = if final_ok { "OK" } else { "NG" }.to_string();
    let reason = if final_ok {
        "全部规则通过".to_string()
    } else if failed_rules.is_empty() {
        "没有可聚合规则".to_string()
    } else {
        failed_rules
            .iter()
            .map(|rule| format!("{}: {}", rule.rule_id, rule.reason))
            .collect::<Vec<_>>()
            .join("; ")
    };
    let final_value = FinalResult {
        final_ok,
        level: if final_ok {
            QualityLevel::Ok
        } else {
            QualityLevel::Ng
        },
        status: status.clone(),
        reason,
        failed_rules,
        all_rules: rules,
        summary: serde_json::json!({ "status": status }),
    };
    Ok(BTreeMap::from([
        ("final".to_string(), Value::Final(final_value)),
        ("status".to_string(), Value::Text(status)),
        ("passed".to_string(), Value::Bool(final_ok)),
    ]))
}

fn rule(
    ctx: &NodeContext<'_>,
    ok: bool,
    actual: serde_json::Value,
    expected: serde_json::Value,
    reason: &str,
    source_field: Option<String>,
) -> RuleResult {
    RuleResult {
        rule_id: text(&ctx.node.params, "rule_id", &ctx.node.id),
        ok,
        level: if ok {
            QualityLevel::Ok
        } else {
            QualityLevel::Ng
        },
        reason: reason.to_string(),
        actual,
        expected,
        source_node: Some(ctx.node.id.clone()),
        source_field,
    }
}

fn rule_out(rule: RuleResult) -> Result<BTreeMap<String, Value>> {
    Ok(BTreeMap::from([
        ("passed".to_string(), Value::Bool(rule.ok)),
        ("rule".to_string(), Value::Rule(rule)),
    ]))
}

fn collect_rules(ctx: &NodeContext<'_>) -> Vec<RuleResult> {
    let mut rules = Vec::new();
    for value in ctx.inputs.values() {
        match value {
            Value::Rule(rule) => rules.push(rule.clone()),
            Value::Rules(items) => rules.extend(items.clone()),
            Value::Final(final_result) => rules.extend(final_result.all_rules.clone()),
            Value::Bool(ok) => rules.push(RuleResult {
                rule_id: "bool_input".to_string(),
                ok: *ok,
                level: if *ok {
                    QualityLevel::Ok
                } else {
                    QualityLevel::Ng
                },
                reason: if *ok {
                    "布尔输入通过"
                } else {
                    "布尔输入未通过"
                }
                .to_string(),
                actual: serde_json::Value::from(*ok),
                expected: serde_json::Value::from(true),
                source_node: None,
                source_field: None,
            }),
            _ => {}
        }
    }
    rules
}

fn value_number(value: Option<&Value>, field: &str) -> Option<f64> {
    match value? {
        Value::Json(json) => json_path(json, field).and_then(serde_json::Value::as_f64),
        Value::Measurements(set) => {
            if field.is_empty() {
                set.measurements.first().and_then(|item| item.value_px)
            } else {
                set.measurements
                    .iter()
                    .find(|item| item.name == field)
                    .and_then(|item| item.value_mm.or(item.value_px))
            }
        }
        Value::Contours(set) => {
            let first = set.contours.first()?;
            match field {
                "count" => Some(set.count as f64),
                "area_px" => Some(first.area_px),
                "area_mm2" => first.area_mm2,
                "perimeter_px" => first.perimeter_px,
                "perimeter_mm" => first.perimeter_mm,
                "roundness" => first.roundness,
                "aspect_ratio" => first.aspect_ratio,
                "width_px" => Some(first.bbox.width),
                "height_px" => Some(first.bbox.height),
                _ => None,
            }
        }
        Value::Detections(detections) => match field {
            "count" => Some(detections.len() as f64),
            "confidence" | "score" => detections.first().map(|item| item.score),
            _ => None,
        },
        Value::Classifications(items) => match field {
            "count" => Some(items.len() as f64),
            "confidence" | "score" => items.first().map(|item| item.score),
            _ => None,
        },
        Value::Segmentations(items) => match field {
            "count" => Some(items.len() as f64),
            "confidence" | "score" => items.first().map(|item| item.score),
            "area_px" => items.first().map(|item| item.area_px),
            _ => None,
        },
        Value::Poses(items) => match field {
            "count" => Some(items.len() as f64),
            "confidence" | "score" => items.first().map(|item| item.score),
            _ => None,
        },
        Value::OrientedBoxes(items) => match field {
            "count" => Some(items.len() as f64),
            "confidence" | "score" => items.first().map(|item| item.score),
            "angle_deg" => items.first().map(|item| item.angle_deg),
            _ => None,
        },
        Value::Location(location) => match field {
            "score" | "confidence" => location.score,
            "x" => Some(location.x),
            "y" => Some(location.y),
            "angle_deg" => location.angle_deg,
            _ => None,
        },
        Value::Bool(ok) => Some(if *ok { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn value_bool(value: Option<&Value>, field: &str) -> Option<bool> {
    match value? {
        Value::Bool(ok) => Some(*ok),
        Value::Rule(rule) => Some(rule.ok),
        Value::Final(result) => Some(result.final_ok),
        Value::Json(json) => json_path(json, field).and_then(serde_json::Value::as_bool),
        Value::Location(location) if field == "found" || field.is_empty() => Some(location.found),
        _ => None,
    }
}

fn value_text(value: Option<&Value>, field: &str) -> Option<String> {
    match value? {
        Value::Text(text) => Some(text.clone()),
        Value::Json(json) => json_path(json, field)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        Value::Final(result) if field == "status" || field.is_empty() => {
            Some(result.status.clone())
        }
        _ => None,
    }
}

fn value_count(value: Option<&Value>) -> usize {
    match value {
        Some(Value::Regions(items)) => items.len(),
        Some(Value::Contours(items)) => items.count,
        Some(Value::Detections(items)) => items.len(),
        Some(Value::Classifications(items)) => items.len(),
        Some(Value::Segmentations(items)) => items.len(),
        Some(Value::Poses(items)) => items.len(),
        Some(Value::OrientedBoxes(items)) => items.len(),
        Some(Value::Measurements(items)) => items.measurements.len(),
        Some(Value::Rules(items)) => items.len(),
        Some(Value::Json(json)) => {
            json.get("count")
                .or_else(|| json.get("contour_count"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_else(|| json.as_array().map_or(0, Vec::len) as u64) as usize
        }
        Some(Value::Bool(ok)) => usize::from(*ok),
        _ => 0,
    }
}

fn classes(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Detections(detections)) => {
            detections.iter().map(|item| item.label.clone()).collect()
        }
        Some(Value::Classifications(items)) => {
            items.iter().map(|item| item.label.clone()).collect()
        }
        Some(Value::Segmentations(items)) => items.iter().map(|item| item.label.clone()).collect(),
        Some(Value::Poses(items)) => items.iter().map(|item| item.label.clone()).collect(),
        Some(Value::OrientedBoxes(items)) => items.iter().map(|item| item.label.clone()).collect(),
        Some(Value::Json(json)) => json_path(json, "class")
            .or_else(|| json_path(json, "label"))
            .and_then(serde_json::Value::as_str)
            .map(|item| vec![item.to_string()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn boxes(value: Option<&Value>) -> Vec<(f64, f64, f64, f64)> {
    match value {
        Some(Value::Contours(set)) => set
            .contours
            .iter()
            .map(|item| (item.bbox.x, item.bbox.y, item.bbox.width, item.bbox.height))
            .collect(),
        Some(Value::Regions(items)) => items
            .iter()
            .map(|item| (item.bbox.x, item.bbox.y, item.bbox.width, item.bbox.height))
            .collect(),
        Some(Value::Detections(items)) => items
            .iter()
            .map(|item| (item.bbox.x, item.bbox.y, item.bbox.width, item.bbox.height))
            .collect(),
        Some(Value::Segmentations(items)) => items
            .iter()
            .map(|item| (item.bbox.x, item.bbox.y, item.bbox.width, item.bbox.height))
            .collect(),
        Some(Value::Poses(items)) => items
            .iter()
            .map(|item| (item.bbox.x, item.bbox.y, item.bbox.width, item.bbox.height))
            .collect(),
        Some(Value::OrientedBoxes(items)) => items
            .iter()
            .map(|item| {
                (
                    item.cx - item.width / 2.0,
                    item.cy - item.height / 2.0,
                    item.width,
                    item.height,
                )
            })
            .collect(),
        Some(Value::Location(location)) => location
            .bbox
            .as_ref()
            .map(|bbox| vec![(bbox.x, bbox.y, bbox.width, bbox.height)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn json_path<'a>(json: &'a serde_json::Value, field: &str) -> Option<&'a serde_json::Value> {
    if field.is_empty() {
        return Some(json);
    }
    let mut current = json;
    for part in field.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn optional_number(value: &serde_json::Value, key: &str) -> Option<f64> {
    value.get(key).and_then(serde_json::Value::as_f64)
}

fn number(value: &serde_json::Value, key: &str, default: f64) -> f64 {
    value
        .get(key)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(default)
}

fn bool_param(value: &serde_json::Value, key: &str, default: bool) -> bool {
    value
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(default)
}

fn text(value: &serde_json::Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or(default)
        .to_string()
}
