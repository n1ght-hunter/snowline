//! State management for bar graphs

use crate::{utils::ZoomableGraphState, zoom::Zoom};

#[derive(Debug, Clone)]
pub struct BarGraphState {
    pub zoom: Zoom,
    pub hovered_bar: Option<usize>,
}

impl BarGraphState {
    pub fn new(initial_zoom: Zoom) -> Self {
        Self {
            zoom: initial_zoom,
            hovered_bar: None,
        }
    }
}

impl Default for BarGraphState {
    fn default() -> Self {
        Self {
            zoom: Zoom::default(),
            hovered_bar: None,
        }
    }
}

impl ZoomableGraphState for BarGraphState {
    fn zoom(&self) -> Zoom {
        self.zoom
    }

    fn set_zoom(&mut self, zoom: Zoom) {
        self.zoom = zoom;
    }
}
