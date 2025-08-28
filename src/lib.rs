//! A library for creating pre-made graphs in Iced.
//!
//!

pub mod bar_graph;
pub mod line_graph;
pub mod zoom;

pub mod prelude {
    pub use crate::bar_graph::{BarGraph, BarGraphState};
    pub use crate::line_graph::{LineGraph, LineGraphState};
    pub use crate::zoom::Zoom;
}
