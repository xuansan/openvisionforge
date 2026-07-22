use std::collections::BTreeMap;

use crate::{OperatorSchema, Result, VisionError};

#[derive(Default)]
pub struct OperatorRegistry {
    operators: BTreeMap<String, OperatorSchema>,
}

impl OperatorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, schema: OperatorSchema) {
        self.operators.insert(schema.id.to_string(), schema);
    }

    pub fn get(&self, id: &str) -> Result<&OperatorSchema> {
        self.operators
            .get(id)
            .ok_or_else(|| VisionError::UnknownOperator(id.to_string()))
    }

    pub fn list(&self) -> impl Iterator<Item = &OperatorSchema> {
        self.operators.values()
    }
}
