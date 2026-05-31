pub(crate) mod current_time;
pub(crate) mod file_polling;
pub(crate) mod temperature;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
pub use current_time::CurrentTimeService;
pub use temperature::TemperatureService;

pub trait OverlayService {
    fn get_text(&self) -> String;

    fn start_service(&mut self, _is_running: Arc<AtomicBool>) { }
}