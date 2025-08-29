//! Shared interaction types for all graph types

use crate::zoom::Zoom;

/// Generic graph interaction events
#[derive(Debug, Clone)]
pub enum GraphInteraction<T> {
    /// Item hovered with index
    ItemHovered(usize),
    /// Item clicked with index
    ItemClicked(usize),
    /// Zoom level changed
    ZoomChanged(Zoom),
    /// Custom graph-specific interaction
    Custom(T),
}

/// Bar graph specific interaction types
#[derive(Debug, Clone)]
pub enum BarInteraction {
    BarHovered(usize),
    BarClicked(usize),
    ZoomChanged(Zoom),
}

impl From<BarInteraction> for GraphInteraction<BarInteraction> {
    fn from(interaction: BarInteraction) -> Self {
        match interaction {
            BarInteraction::BarHovered(index) => GraphInteraction::ItemHovered(index),
            BarInteraction::BarClicked(index) => GraphInteraction::ItemClicked(index),
            BarInteraction::ZoomChanged(zoom) => GraphInteraction::ZoomChanged(zoom),
        }
    }
}

/// Line graph specific interaction types
#[derive(Debug, Clone)]
pub enum LineInteraction {
    PointHovered(usize),
    PointClicked(usize),
    ZoomChanged(Zoom),
}

impl From<LineInteraction> for GraphInteraction<LineInteraction> {
    fn from(interaction: LineInteraction) -> Self {
        match interaction {
            LineInteraction::PointHovered(index) => GraphInteraction::ItemHovered(index),
            LineInteraction::PointClicked(index) => GraphInteraction::ItemClicked(index),
            LineInteraction::ZoomChanged(zoom) => GraphInteraction::ZoomChanged(zoom),
        }
    }
}
