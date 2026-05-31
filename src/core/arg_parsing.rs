use clap::ArgAction;
use clap::Parser;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum OverlayType {
    Time,
    Temperature { sensor_id: String },
}

impl FromStr for OverlayType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "time" {
            Ok(OverlayType::Time)
        } else if let Some(sensor_id) = s.strip_prefix("temperature=") {
            if sensor_id.is_empty() {
                Err(
                    "Temperature overlay requires a sensor_id (e.g., temperature=sensor_1)"
                        .to_string(),
                )
            } else {
                Ok(OverlayType::Temperature {
                    sensor_id: sensor_id.to_string(),
                })
            }
        } else {
            Err(format!("Unknown overlay type: '{}'.", s))
        }
    }
}

#[derive(Parser)]
#[command(version, about = "Monitor smart displays with live camera streams, motion detection, and text overlays")]
pub struct CliArgs {
    #[arg(required = true)]
    pub channel_urls: Vec<String>,

    #[arg(short, long, default_value_t = false)]
    pub gpu: bool,

    #[arg(long, default_value_t = false)]
    pub benchmark: bool,

    #[arg(long = "overlay", action = ArgAction::Append)]
    pub overlays: Vec<OverlayType>,
}
