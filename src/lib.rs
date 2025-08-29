//! A library for creating pre-made graphs in Iced.
//!
//!

pub mod bar_graph;
pub mod line_graph;
pub mod utils;
pub mod zoom;

pub mod prelude {
    pub use crate::{
        bar_graph::{
            BarGraph,
            color_scheme::{BarColorParams, BarColorScheme},
            state::BarGraphState,
        },
        line_graph::{
            LineGraph,
            color_scheme::{PointColorParams, PointColorScheme},
            state::LineGraphState,
        },
        utils::{
            BarInteraction, GraphInteraction, GridConfig, LineInteraction, ZoomableGraphState,
            calculate_visible_range, draw_average_line, draw_grid, draw_y_axis_labels,
        },
        zoom::Zoom,
    };
}
