mod image_save;
mod report_csv;
mod result_json;
mod trace;

use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    image_save::register(registry);
    result_json::register(registry);
    trace::register(registry);
    report_csv::register(registry);
}
