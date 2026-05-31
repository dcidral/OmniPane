use crate::services::OverlayService;
use chrono::Utc;

pub struct CurrentTimeService {}

impl CurrentTimeService {
    pub fn new() -> Self {
        Self {}
    }
    
    fn get_current_time(&self) -> String {
        Utc::now()
            .format("%d/%m/%Y %H:%M:%S")
            .to_string()
    }
}


impl OverlayService for CurrentTimeService {
    fn get_text(&self) -> String {
        self.get_current_time()
    }
}
