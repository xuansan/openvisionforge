#![allow(dead_code)]

#[derive(Debug, Clone)]
pub(crate) struct RknnConfig {
    pub model_path: String,
    pub core_mask: String,
}

impl RknnConfig {
    pub(crate) fn reserved(model_path: String) -> Self {
        Self {
            model_path,
            core_mask: "auto".to_string(),
        }
    }
}
