pub use canvas::Cache;
use iced::{Color, Event, Rectangle, Renderer, Theme, mouse, widget::canvas};

use crate::zoom::Zoom;

// Make modules public for prelude access, but don't re-export types here
pub mod color_scheme;
use color_scheme::{BarColorParams, BarColorScheme};

pub mod state;
use state::BarGraphState;

// Import drawing utilities
mod drawing;

// Re-export the shared interaction type for backward compatibility
pub use crate::utils::BarInteraction as Interaction;

#[allow(missing_debug_implementations)]
pub struct BarGraph<'a, I, T>
where
    I: Iterator<Item = T> + Clone + 'a,
{
    pub datapoints: I,
    pub cache: &'a canvas::Cache,
    pub bar_color: Option<Color>,
    pub bar_width: f32,
    pub show_grid: bool,
    pub show_labels: bool,
    pub zoom: Zoom,
    pub base_bars: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub bar_color_scheme: BarColorScheme,
    pub to_float: fn(T) -> f64,
    pub external_zoom: Option<Zoom>, // Optional external zoom override
}

impl<'a, I, T> BarGraph<'a, I, T>
where
    I: Iterator<Item = T> + Clone + 'a,
    T: Copy + Into<f64>,
{
    pub fn new(datapoints: I, cache: &'a canvas::Cache, to_float: fn(T) -> f64) -> Self {
        Self {
            datapoints,
            cache,
            bar_color: None,
            bar_width: 2.0,
            show_grid: true,
            show_labels: true,
            zoom: Zoom::default(),
            base_bars: 50.0,
            zoom_min: 0.1,
            zoom_max: 10.0,
            bar_color_scheme: BarColorScheme::default(),
            to_float,
            external_zoom: None,
        }
    }

    /// Get the initial state for this BarGraph
    pub fn initial_state(&self) -> BarGraphState {
        BarGraphState::new(self.zoom)
    }

    /// Set external zoom (overrides internal state zoom)
    pub fn external_zoom(mut self, zoom: Zoom) -> Self {
        self.external_zoom = Some(zoom);
        self
    }

    /// Clear external zoom (use internal state zoom)
    pub fn use_internal_zoom(mut self) -> Self {
        self.external_zoom = None;
        self
    }

    pub fn bar_color(mut self, color: Color) -> Self {
        self.bar_color = Some(color);
        self
    }

    pub fn bar_width(mut self, width: f32) -> Self {
        self.bar_width = width;
        self
    }

    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    pub fn base_bars(mut self, bars: f32) -> Self {
        self.base_bars = bars;
        self
    }

    pub fn zoom_range(mut self, min: f32, max: f32) -> Self {
        self.zoom_min = min;
        self.zoom_max = max;
        self
    }

    pub fn zoom_min(mut self, min: f32) -> Self {
        self.zoom_min = min;
        self
    }

    pub fn zoom_max(mut self, max: f32) -> Self {
        self.zoom_max = max;
        self
    }

    pub fn bar_color_fn<F>(mut self, color_fn: F) -> Self
    where
        F: Fn(&BarColorParams) -> Color + Send + Sync + 'static,
    {
        self.bar_color_scheme = BarColorScheme::new_function(color_fn);
        self
    }

    pub fn single_bar_color(mut self, color: Color) -> Self {
        self.bar_color_scheme = BarColorScheme::new_single(color);
        self
    }

    pub fn bar_color_scheme(mut self, scheme: BarColorScheme) -> Self {
        self.bar_color_scheme = scheme;
        self
    }

    /// Default performance-based color scheme (green for good, red for poor, orange for average)
    pub fn performance_colors(mut self) -> Self {
        self.bar_color_scheme = BarColorScheme::performance();
        self
    }

    /// Theme-aware color scheme that adapts to the current theme
    pub fn theme_colors(mut self) -> Self {
        self.bar_color_scheme = BarColorScheme::theme_colors();
        self
    }

    /// Index-based color scheme: green for index < 6, yellow for index 6, red for index > 6
    pub fn traffic_light_colors(mut self) -> Self {
        self.bar_color_scheme = BarColorScheme::traffic_light();
        self
    }

    /// Get the effective zoom value (external zoom if set, otherwise state zoom)
    fn effective_zoom(&self, state: &BarGraphState) -> Zoom {
        self.external_zoom.unwrap_or(state.zoom)
    }
}

impl<'a, I, T> canvas::Program<Interaction> for BarGraph<'a, I, T>
where
    I: Iterator<Item = T> + Clone + 'a,
    T: Copy + Into<f64>,
{
    type State = BarGraphState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Interaction>> {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    let effective_zoom = self.effective_zoom(state);
                    let visible_bars = match effective_zoom {
                        Zoom::Full => self.datapoints.clone().count(),
                        Zoom::Value(zoom_value) => (self.base_bars * zoom_value) as usize,
                    };
                    let bar_width = bounds.width / visible_bars as f32;

                    let bar_index = (cursor_position.x / bar_width) as usize;

                    if bar_index < visible_bars {
                        if state.hovered_bar != Some(bar_index) {
                            state.hovered_bar = Some(bar_index);
                            return Some(canvas::Action::publish(Interaction::BarHovered(
                                bar_index,
                            )));
                        }
                    } else if state.hovered_bar.is_some() {
                        state.hovered_bar = None;
                    }
                } else if state.hovered_bar.is_some() {
                    state.hovered_bar = None;
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(bar_index) = state.hovered_bar {
                    Some(canvas::Action::publish(Interaction::BarClicked(bar_index)))
                } else {
                    None
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if cursor.is_over(bounds) {
                    let new_zoom = match delta {
                        mouse::ScrollDelta::Lines { y, .. }
                        | mouse::ScrollDelta::Pixels { y, .. } => {
                            if *y > 0.0 {
                                state.zoom.increment_with_limits(self.zoom_max)
                            } else {
                                state.zoom.decrement_with_limits(self.zoom_min)
                            }
                        }
                    };

                    if new_zoom != state.zoom {
                        state.zoom = new_zoom;
                        return Some(canvas::Action::publish(Interaction::ZoomChanged(new_zoom)));
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
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let cursor = cursor.position_in(bounds);
            let bounds = frame.size();

            let effective_zoom = self.effective_zoom(state);
            let visible_bars = match effective_zoom {
                Zoom::Full => self.datapoints.clone().count(),
                Zoom::Value(zoom_value) => (self.base_bars * zoom_value) as usize,
            };

            // Enumerate datapoints to get index
            let datapoints: Vec<(usize, T)> = self.datapoints.clone().enumerate().collect();

            if datapoints.is_empty() {
                return;
            }

            // Calculate average for color scheme
            let values: Vec<f64> = datapoints
                .iter()
                .map(|(_, v)| (self.to_float)(*v))
                .collect();
            let average = values.iter().sum::<f64>() / values.len() as f64;

            // Find max value for scaling
            let max_value = values.iter().fold(0.0f64, |a, &b| a.max(b));
            if max_value == 0.0 {
                return;
            }

            // Draw all components using the modular functions
            self.draw_bars(
                frame,
                bounds,
                visible_bars,
                &datapoints,
                average,
                max_value,
                theme,
            );
            self.draw_grid_and_scale(frame, bounds, visible_bars, max_value, theme);
            self.draw_average_line(frame, bounds, average, max_value);
            self.draw_bar_labels_and_hover(frame, bounds, visible_bars, &datapoints, cursor, theme);
        });

        vec![geometry]
    }
}
