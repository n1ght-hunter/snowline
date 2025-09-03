//! State management for line graphs

use crate::{utils::ZoomableGraphState, zoom::Zoom};

#[derive(Debug, Clone)]
pub enum PanMode {
    Start,
    End,
    Absolute(usize),
}

#[derive(Debug, Clone)]
pub struct Pan {
    pub mode: PanMode,
}

impl Default for Pan {
    fn default() -> Self {
        // Default to following the end (most recent) when zoomed
        Self { mode: PanMode::End }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LineGraphState {
    pub zoom: Zoom,
    pub hovered_point: Option<usize>,
    pub pan: Pan,         // logical pan mode
    pub shift_down: bool, // track Shift for pan-only scroll
}

impl LineGraphState {
    pub fn new(initial_zoom: Zoom) -> Self {
        Self {
            zoom: initial_zoom,
            hovered_point: None,
            pan: Pan::default(),
            shift_down: false,
        }
    }

    pub fn pan_start(&mut self) {
        self.pan.mode = PanMode::Start;
    }

    pub fn pan_end(&mut self) {
        self.pan.mode = PanMode::End;
    }

    pub fn pan_absolute(&mut self, start: usize) {
        self.pan.mode = PanMode::Absolute(start);
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
