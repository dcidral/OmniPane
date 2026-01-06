pub(crate) mod time_provider;
pub(crate) mod file_polling;
pub(crate) mod temperature_provider;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
pub use time_provider::TimeOverlayTextProvider;
pub use temperature_provider::TemperatureOverlayTextProvider;

pub trait OverlayTextProvider {
    fn get_text(&self) -> String;

    fn start_service(&mut self, _is_running: Arc<AtomicBool>) { }
}