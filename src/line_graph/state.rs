//! State management for line graphs

use crate::{utils::ZoomableGraphState, zoom::Zoom};

#[derive(Debug, Clone)]
pub struct LineGraphState {
    pub zoom: Zoom,
    pub hovered_point: Option<usize>,
}

impl LineGraphState {
    pub fn new(initial_zoom: Zoom) -> Self {
        Self {
            zoom: initial_zoom,
            hovered_point: None,
        }
    }
}

impl Default for LineGraphState {
    fn default() -> Self {
        Self {
            zoom: Zoom::default(),
            hovered_point: None,
        }
    }
}

impl ZoomableGraphState for LineGraphState {
    fn zoom(&self) -> Zoom {
        self.zoom
    }

    fn set_zoom(&mut self, zoom: Zoom) {
        self.zoom = zoom;
    }
}
