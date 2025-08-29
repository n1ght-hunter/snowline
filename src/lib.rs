//! A library for creating pre-made graphs in Iced.
//!
//!

pub mod bar_graph;
pub mod line_graph;
pub mod zoom;
pub mod utils;

pub mod prelude {
    pub use crate::{
        bar_graph::{
            BarGraph,
            state::BarGraphState,
            color_scheme::{BarColorParams, BarColorScheme},
        },
        line_graph::{
            LineGraph,
            state::LineGraphState,
            color_scheme::{PointColorParams, PointColorScheme},
        },
        zoom::Zoom,
        utils::{
            GraphInteraction, BarInteraction, LineInteraction,
            GridConfig, draw_grid, draw_y_axis_labels, draw_average_line,
            ZoomableGraphState, calculate_visible_range,
        },
    };
}
