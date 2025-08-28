//! A library for creating pre-made graphs in Iced.
//!
//!

pub mod line_graph;
pub mod zoom;

pub mod prelude {
    pub use crate::line_graph::{Interaction, LineGraph, LineGraphState};
    pub use crate::zoom::Zoom;
}
