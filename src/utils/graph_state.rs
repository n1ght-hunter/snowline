//! Shared graph state functionality

use crate::zoom::Zoom;

/// Common trait for graph state that supports zoom
pub trait ZoomableGraphState {
    fn zoom(&self) -> Zoom;
    fn set_zoom(&mut self, zoom: Zoom);
    fn effective_zoom(&self, external_zoom: Option<Zoom>) -> Zoom {
        external_zoom.unwrap_or(self.zoom())
    }
}

/// Helper methods for calculating visible data points based on zoom
pub fn calculate_visible_range(total_items: usize, zoom: Zoom, base_items: f32) -> (usize, usize) {
    let visible_count = match zoom {
        Zoom::Full => total_items,
        Zoom::Value(zoom_factor) => {
            if zoom_factor >= 1.0 {
                // Zooming in: show fewer items for more detail
                ((base_items / zoom_factor).max(5.0) as usize).min(total_items)
            } else {
                // Zooming out: show more items for broader view
                ((base_items / zoom_factor) as usize).min(total_items)
            }
        }
    };

    let start_index = match zoom {
        Zoom::Full => 0, // Show all data from the beginning
        Zoom::Value(_) => {
            if total_items > visible_count {
                // Show the most recent data
                total_items - visible_count
            } else {
                0
            }
        }
    };

    (start_index, visible_count)
}
