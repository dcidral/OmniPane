pub mod time_provider;
pub use time_provider::TimeOverlayTextProvider;

pub trait OverlayTextProvider {
    fn get_text(&self) -> String;
}