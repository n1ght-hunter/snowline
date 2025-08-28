//! Common types used by line graphs

use iced::{Color, Theme};
use crate::line_graph::zoom::Zoom;

#[derive(Debug, Clone)]
pub enum Interaction {
    PointHovered(usize),
    PointClicked(usize),
    ZoomChanged(Zoom),
}

pub enum PointColorScheme {
    Single(Color),
    Function(Box<dyn Fn(f64, f64, &Theme) -> Color + Send + Sync>),
}

impl Default for PointColorScheme {
    fn default() -> Self {
        // Default performance-based color scheme
        Self::Function(Box::new(|value, average, _theme| {
            if value < average * 0.7 {
                Color::from_rgb(0.2, 0.8, 0.3) // Green for good performance
            } else if value > average * 1.3 {
                Color::from_rgb(0.9, 0.3, 0.3) // Red for poor performance
            } else {
                Color::from_rgb(1.0, 0.7, 0.2) // Orange for average performance
            }
        }))
    }
}
