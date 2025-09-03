//! Shared label formatting trait for graphs

use crate::zoom::Zoom;

/// Abstraction for formatting numeric values and titles/subtitles
/// so graphs can customize units and precision consistently.
pub trait LabelFormatter: Send + Sync {
    fn format_y_axis(&self, value: f64) -> String;
    fn format_tooltip(&self, value: f64) -> String;
    fn format_average_text(&self, value: f64) -> String;
    fn format_title(&self, zoom: Zoom) -> Option<String>;
    fn format_subtitle(&self, zoom: Zoom, start_idx: usize, end_idx: usize, count: usize)
    -> String;
}
