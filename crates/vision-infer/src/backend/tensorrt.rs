#![allow(dead_code)]

#[derive(Debug, Clone)]
pub(crate) struct TensorRtConfig {
    pub engine_path: String,
    pub device_id: i64,
    pub fp16: bool,
}

impl TensorRtConfig {
    pub(crate) fn reserved(engine_path: String) -> Self {
        Self {
            engine_path,
            device_id: 0,
            fp16: true,
        }
    }
}
