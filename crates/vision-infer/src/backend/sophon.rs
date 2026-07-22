#![allow(dead_code)]

#[derive(Debug, Clone)]
pub(crate) struct SophonConfig {
    pub bmodel_path: String,
    pub device_id: i64,
}

impl SophonConfig {
    pub(crate) fn reserved(bmodel_path: String) -> Self {
        Self {
            bmodel_path,
            device_id: 0,
        }
    }
}
