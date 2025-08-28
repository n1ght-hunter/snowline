//! State management and canvas program implementation for line graphs

use iced::mouse;
use iced::widget::canvas;
use iced::{Center, Color, Event, Font, Pixels, Point, Rectangle, Renderer, Right, Size, Theme};

use crate::line_graph::zoom::Zoom;
use crate::line_graph::types::{Interaction, PointColorScheme};
use crate::line_graph::line_graph::LineGraph;

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

impl<'a, I, T> canvas::Program<Interaction> for LineGraph<'a, I, T>
where
    I: Iterator<Item = T> + Clone + 'a,
    T: Copy + Into<f64>,
{
    type State = LineGraphState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Interaction>> {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Only enable hover when zoomed in (above zoom_min) and points are visible
                let effective_zoom = self.effective_zoom(state);
                if effective_zoom.value() > self.zoom_min {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        let new_hovered = self.find_nearest_point(cursor_position, bounds);

                        if state.hovered_point != new_hovered {
                            state.hovered_point = new_hovered;
                            self.cache.clear();

                            if let Some(point_index) = new_hovered {
                                return Some(canvas::Action::publish(Interaction::PointHovered(
                                    point_index,
                                )));
                            } else {
                                return Some(canvas::Action::request_redraw());
                            }
                        }
                    } else if state.hovered_point.is_some() {
                        state.hovered_point = None;
                        self.cache.clear();
                        return Some(canvas::Action::request_redraw());
                    }
                } else {
                    // In full view, clear any existing hover state
                    if state.hovered_point.is_some() {
                        state.hovered_point = None;
                        self.cache.clear();
                        return Some(canvas::Action::request_redraw());
                    }
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some(point_index) = self.find_nearest_point(cursor_position, bounds) {
                        return Some(canvas::Action::publish(Interaction::PointClicked(
                            point_index,
                        )));
                    }
                }
                None
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                // Only handle zoom changes if external zoom is not set
                if self.external_zoom.is_none() {
                    match delta {
                        mouse::ScrollDelta::Lines { y, .. }
                        | mouse::ScrollDelta::Pixels { y, .. } => {
                            let new_zoom = if y.is_sign_positive() {
                                state.zoom.increment_with_limits(self.zoom_max)
                            } else {
                                state
                                    .zoom
                                    .decrement_with_limits(self.zoom_min)
                            };

                            if new_zoom != state.zoom {
                                state.zoom = new_zoom;
                                self.cache.clear();
                                return Some(canvas::Action::publish(Interaction::ZoomChanged(
                                    new_zoom,
                                )));
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let bounds = frame.size();
            let palette = theme.extended_palette();

            // Collect datapoints to work with them (enumerate internally)
            let all_datapoints: Vec<_> = self.datapoints.clone().enumerate().collect();

            if all_datapoints.is_empty() {
                return;
            }

            // Calculate chart dimensions
            let padding = 40.0;
            let chart_width = bounds.width - 2.0 * padding;
            let chart_height = bounds.height - 2.0 * padding;

            // Apply zoom to determine how many points to show
            let effective_zoom = self.effective_zoom(state);
            let zoom_factor = effective_zoom.value();
            let base_points = self.base_points; // Use configurable base points
            let max_visible_points = if zoom_factor >= 1.0 {
                // Zooming in: show fewer points for more detail
                (base_points / zoom_factor).max(5.0) as usize
            } else if zoom_factor <= self.zoom_min {
                // At minimum zoom or below, show all available data points
                all_datapoints.len()
            } else {
                // Zooming out: show more points for broader view
                (base_points / zoom_factor).min(all_datapoints.len() as f32) as usize
            };

            let start_index = if zoom_factor <= self.zoom_min {
                // At minimum zoom, show all data from the beginning
                0
            } else if all_datapoints.len() > max_visible_points {
                // For other zoom levels, show the most recent data
                all_datapoints.len() - max_visible_points
            } else {
                0
            };

            let visible_datapoints: Vec<_> = all_datapoints.into_iter().skip(start_index).collect();

            if visible_datapoints.is_empty() {
                return;
            }

            // Find min/max values for proper scaling from visible data
            let values: Vec<f64> = visible_datapoints
                .iter()
                .map(|(_, v)| (self.to_float)(*v))
                .collect();
            let min_value = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_value = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let value_range = max_value - min_value;

            if value_range == 0.0 {
                return; // Avoid division by zero
            }

            let average = values.iter().sum::<f64>() / values.len() as f64;

            // Create points for the line (using visible datapoints)
            let points: Vec<Point> = visible_datapoints
                .iter()
                .enumerate()
                .map(|(i, (_, value))| {
                    let x = padding
                        + (i as f32 / (visible_datapoints.len() - 1).max(1) as f32) * chart_width;
                    let value_f64 = (self.to_float)(*value);
                    let normalized_value = (value_f64 - min_value) / value_range;
                    let y = padding + chart_height - (normalized_value as f32 * chart_height);
                    Point::new(x, y)
                })
                .collect();

            // Draw grid if enabled
            if self.show_grid {
                self.draw_grid(frame, padding, chart_width, chart_height, &palette);
            }

            // Draw the line
            if points.len() > 1 {
                self.draw_line(frame, &points, &palette);
            }

            // Draw data points if enabled (but not in full view)
            if self.show_points && zoom_factor > self.zoom_min {
                self.draw_points(
                    frame,
                    &points,
                    &values,
                    &state.hovered_point,
                    average,
                    theme,
                );
            }

            // Draw labels if enabled
            if self.show_labels {
                // Convert visible datapoints to (usize, f64) for the draw_labels method
                let visible_datapoints_f64: Vec<(usize, f64)> = visible_datapoints
                    .iter()
                    .map(|(i, v)| (*i, (self.to_float)(*v)))
                    .collect();

                self.draw_labels(
                    frame,
                    bounds,
                    padding,
                    chart_width,
                    chart_height,
                    min_value,
                    max_value,
                    average,
                    value_range,
                    &palette,
                    zoom_factor,
                    &visible_datapoints_f64,
                );
            }
        });

        vec![geometry]
    }
}
