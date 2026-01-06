use crate::overlay_text_providers::file_polling::FilePoller;
use crate::overlay_text_providers::OverlayTextProvider;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

pub struct TemperatureOverlayTextProvider {
    file_poller: FilePoller,
}

impl TemperatureOverlayTextProvider {
    pub fn new(sensor_id: &str) -> Self {
        let sensor_file_path = format!("/sys/bus/w1/devices/{}/w1_slave", sensor_id);
        let poll_interval = Duration::from_secs(5);

        Self {
            file_poller: FilePoller::new(sensor_file_path, poll_interval),
        }
    }

    pub fn get_temperature_text(&self) -> String {
        match self
            .file_poller
            .get_current_file_content()
            .split("t=")
            .last()
        {
            Some(value) => match value.trim().to_string().parse::<f32>() {
                Ok(raw_temp) => {
                    let temperature = raw_temp / 1000.0;
                    format!("Temp.: {:.1} °C", temperature)
                }
                Err(_) => {
                    format!("Error parsing temperature {}", value)
                }
            },
            None => "No temperature found".to_string(),
        }
    }
}

impl OverlayTextProvider for TemperatureOverlayTextProvider {
    fn get_text(&self) -> String {
        self.get_temperature_text()
    }

    fn start_service(&mut self, is_running: Arc<AtomicBool>) {
        self.file_poller.start(is_running);
    }
}
