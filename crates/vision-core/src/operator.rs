use std::collections::BTreeMap;

use crate::{Result, Value};

pub type OperatorFn = fn(&crate::NodeContext<'_>) -> Result<BTreeMap<String, Value>>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ParamSchema {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: ParamKind,
    pub required: bool,
    pub default: Option<&'static str>,
    pub description: &'static str,
    pub options: &'static [&'static str],
    pub group: Option<&'static str>,
    pub order: i32,
    pub advanced: bool,
    pub unit: Option<&'static str>,
    pub placeholder: Option<&'static str>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub accept: &'static [&'static str],
    pub deployment_required: bool,
    pub overridable: bool,
    pub visible_if: Option<&'static str>,
    pub validation_regex: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum ParamKind {
    String,
    Number,
    Boolean,
    Json,
    FilePath,
    FolderPath,
    Url,
    Secret,
    Enum,
}

impl ParamSchema {
    pub const fn new(
        key: &'static str,
        label: &'static str,
        kind: ParamKind,
        required: bool,
        default: Option<&'static str>,
        description: &'static str,
        options: &'static [&'static str],
    ) -> Self {
        Self {
            key,
            label,
            kind,
            required,
            default,
            description,
            options,
            group: None,
            order: 0,
            advanced: false,
            unit: None,
            placeholder: None,
            min: None,
            max: None,
            step: None,
            accept: &[],
            deployment_required: required,
            overridable: true,
            visible_if: None,
            validation_regex: None,
        }
    }

    pub const fn group(mut self, group: &'static str) -> Self {
        self.group = Some(group);
        self
    }

    pub const fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub const fn advanced(mut self, advanced: bool) -> Self {
        self.advanced = advanced;
        self
    }

    pub const fn unit(mut self, unit: &'static str) -> Self {
        self.unit = Some(unit);
        self
    }

    pub const fn placeholder(mut self, placeholder: &'static str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub const fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub const fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub const fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    pub const fn range(mut self, min: f64, max: f64, step: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self.step = Some(step);
        self
    }

    pub const fn accept(mut self, accept: &'static [&'static str]) -> Self {
        self.accept = accept;
        self
    }

    pub const fn deployment_required(mut self, required: bool) -> Self {
        self.deployment_required = required;
        self
    }

    pub const fn overridable(mut self, overridable: bool) -> Self {
        self.overridable = overridable;
        self
    }

    pub const fn visible_if(mut self, expression: &'static str) -> Self {
        self.visible_if = Some(expression);
        self
    }

    pub const fn validation_regex(mut self, regex: &'static str) -> Self {
        self.validation_regex = Some(regex);
        self
    }
}

#[derive(Debug, Clone)]
pub struct OperatorSchema {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub inputs: &'static [&'static str],
    pub optional_inputs: &'static [&'static str],
    pub outputs: &'static [&'static str],
    pub params: &'static [ParamSchema],
    pub run: OperatorFn,
}
