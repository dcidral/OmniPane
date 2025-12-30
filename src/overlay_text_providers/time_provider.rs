use crate::overlay_text_providers::OverlayTextProvider;
use chrono::Utc;

pub struct TimeOverlayTextProvider {}

impl TimeOverlayTextProvider {
    pub fn new() -> Self {
        Self {}
    }
    
    fn get_current_time(&self) -> String {
        Utc::now()
            .format("%d/%m/%Y %H:%M:%S")
            .to_string()
    }
}


impl OverlayTextProvider for TimeOverlayTextProvider {
    fn get_text(&self) -> String {
        self.get_current_time()
    }
}
