//! A library for creating pre-made graphs in Iced.
//!
//!

pub mod bar_graph;
pub mod line_graph;
pub mod zoom;

pub mod prelude {
    pub use crate::{
        bar_graph::{
            BarGraph, BarGraphState,
            color_scheme::{BarColorParams, BarColorScheme},
        },
        line_graph::{
            LineGraph, LineGraphState,
            color_scheme::{PointColorParams, PointColorScheme},
        },
        zoom::Zoom,
    };
}
