use iced::mouse;
use iced::widget::canvas;
use iced::{
    Bottom, Center, Color, Event, Font, Pixels, Point, Rectangle, Renderer, Right, Size, Theme, Top,
};

pub use canvas::Cache;

// Re-export Zoom for backward compatibility
pub use crate::zoom::Zoom;

#[derive(Debug, Clone)]
pub enum Interaction {
    BarHovered(usize),
    BarClicked(usize),
    ZoomChanged(Zoom),
}

/// Parameters passed to bar color functions
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BarColorParams<'a> {
    pub index: usize,
    pub value: f64,
    pub average: f64,
    pub theme: &'a Theme,
}

pub enum BarColorScheme {
    Single(Color),
    Function(Box<dyn Fn(&BarColorParams) -> Color + Send + Sync>),
}

impl Default for BarColorScheme {
    fn default() -> Self {
        // Default performance-based color scheme
        Self::Function(Box::new(|params| {
            if params.value < params.average * 0.7 {
                Color::from_rgb(0.2, 0.8, 0.3) // Green for good performance
            } else if params.value > params.average * 1.3 {
                Color::from_rgb(0.9, 0.3, 0.3) // Red for poor performance
            } else {
                Color::from_rgb(1.0, 0.7, 0.2) // Orange for average performance
            }
        }))
    }
}

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
        self.bar_color_scheme = BarColorScheme::Function(Box::new(color_fn));
        self
    }

    pub fn single_bar_color(mut self, color: Color) -> Self {
        self.bar_color_scheme = BarColorScheme::Single(color);
        self
    }

    pub fn bar_color_scheme(mut self, scheme: BarColorScheme) -> Self {
        self.bar_color_scheme = scheme;
        self
    }

    /// Default performance-based color scheme (green for good, red for poor, orange for average)
    pub fn performance_colors(mut self) -> Self {
        self.bar_color_scheme = BarColorScheme::Function(Box::new(|params| {
            if params.value < params.average * 0.7 {
                Color::from_rgb(0.2, 0.8, 0.3) // Green for good performance
            } else if params.value > params.average * 1.3 {
                Color::from_rgb(0.9, 0.3, 0.3) // Red for poor performance
            } else {
                Color::from_rgb(1.0, 0.7, 0.2) // Orange for average performance
            }
        }));
        self
    }

    /// Theme-aware color scheme that adapts to the current theme
    pub fn theme_colors(mut self) -> Self {
        self.bar_color_scheme =
            BarColorScheme::Function(Box::new(|params| {
                let palette = params.theme.extended_palette();
                palette.primary.base.color
            }));
        self
    }

    /// Index-based color scheme: green for index < 6, yellow for index 6, red for index > 6
    pub fn traffic_light_colors(mut self) -> Self {
        self.bar_color_scheme = BarColorScheme::Function(Box::new(|params| {
            if params.index < 6 {
                Color::from_rgb(0.2, 0.8, 0.3) // Green
            } else if params.index == 6 {
                Color::from_rgb(1.0, 0.9, 0.0) // Yellow
            } else {
                Color::from_rgb(0.9, 0.3, 0.3) // Red
            }
        }));
        self
    }

    /// Get the effective zoom value (external zoom if set, otherwise state zoom)
    fn effective_zoom(&self, state: &BarGraphState) -> Zoom {
        self.external_zoom.unwrap_or(state.zoom)
    }
}

pub struct BarGraphState {
    pub zoom: Zoom,
    pub hovered_bar: Option<usize>,
}

impl BarGraphState {
    pub fn new(initial_zoom: Zoom) -> Self {
        Self {
            zoom: initial_zoom,
            hovered_bar: None,
        }
    }
}

impl Default for BarGraphState {
    fn default() -> Self {
        Self {
            zoom: Zoom::default(),
            hovered_bar: None,
        }
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
                    let visible_bars = (self.base_bars * effective_zoom.value()) as usize;
                    let bar_width = bounds.width / visible_bars as f32;
                    
                    let bar_index = (cursor_position.x / bar_width) as usize;
                    
                    if bar_index < visible_bars {
                        if state.hovered_bar != Some(bar_index) {
                            state.hovered_bar = Some(bar_index);
                            return Some(canvas::Action::publish(Interaction::BarHovered(bar_index)));
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
                        mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                            if *y > 0.0 {
                                state
                                    .zoom
                                    .increment_with_limits(self.zoom_max)
                            } else {
                                state
                                    .zoom
                                    .decrement_with_limits(self.zoom_min)
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
            let palette = theme.extended_palette();

            let effective_zoom = self.effective_zoom(state);
            let visible_bars = (self.base_bars * effective_zoom.value()) as usize;
            let bar_width = bounds.width / visible_bars as f32;

            // Enumerate datapoints to get index
            let datapoints: Vec<(usize, T)> = self.datapoints.clone().enumerate().collect();

            if datapoints.is_empty() {
                return;
            }

            // Calculate average for color scheme
            let values: Vec<f64> = datapoints.iter().map(|(_, v)| (self.to_float)(*v)).collect();
            let average = values.iter().sum::<f64>() / values.len() as f64;

            // Find max value for scaling
            let max_value = values.iter().fold(0.0f64, |a, &b| a.max(b));
            if max_value == 0.0 {
                return;
            }

            // Reserve space at bottom for zero values and labels
            let bottom_margin = 40.0;
            let available_height = bounds.height - bottom_margin;
            let pixels_per_unit = available_height / max_value as f32;

            // Draw bars
            for (i, (_original_index, datapoint)) in datapoints.iter().take(visible_bars).enumerate() {
                let value = (self.to_float)(*datapoint);
                
                // Minimum bar height for zero values to be visible
                let min_bar_height = if value == 0.0 { 3.0 } else { 0.0 };
                let bar_height = ((value * pixels_per_unit as f64) as f32).max(min_bar_height);

                // Add some padding between bars
                let bar_padding = bar_width * 0.1;
                let actual_bar_width = bar_width - bar_padding;

                let bar = Rectangle {
                    x: i as f32 * bar_width + bar_padding / 2.0,
                    y: bounds.height - bottom_margin - bar_height,
                    width: actual_bar_width,
                    height: bar_height,
                };

                // Determine bar color
                let bar_color = match &self.bar_color_scheme {
                    BarColorScheme::Single(color) => *color,
                    BarColorScheme::Function(color_fn) => {
                        let params = BarColorParams {
                            index: i,
                            value,
                            average,
                            theme,
                        };
                        
                        // For zero values, use a special muted color unless overridden
                        if value == 0.0 {
                            // Let the function decide, but provide a fallback
                            let function_color = color_fn(&params);
                            // If it's the default performance color, use muted instead
                            if function_color == Color::from_rgb(0.2, 0.8, 0.3) || 
                               function_color == Color::from_rgb(0.9, 0.3, 0.3) || 
                               function_color == Color::from_rgb(1.0, 0.7, 0.2) {
                                Color::from_rgb(0.7, 0.7, 0.7)
                            } else {
                                function_color
                            }
                        } else {
                            color_fn(&params)
                        }
                    },
                };

                frame.fill_rectangle(
                    bar.position(),
                    bar.size(),
                    self.bar_color.unwrap_or(bar_color),
                );

                // Draw bar index labels at bottom
                if self.show_labels {
                    frame.fill_text(canvas::Text {
                        content: format!("{}", i),
                        position: Point::new(
                            i as f32 * bar_width + bar_width / 2.0,
                            bounds.height - 5.0
                        ),
                        color: palette.background.base.text.scale_alpha(0.6),
                        size: Pixels(10.0),
                        font: Font::MONOSPACE,
                        align_x: Center.into(),
                        align_y: Bottom,
                        ..canvas::Text::default()
                    });
                }

                // Highlight hovered bar
                let bar_overlay = Rectangle {
                    x: i as f32 * bar_width,
                    y: 0.0,
                    width: bar_width,
                    height: bounds.height - bottom_margin,
                };

                match cursor {
                    Some(cursor_pos) if bar_overlay.contains(cursor_pos) => {
                        frame.fill_rectangle(
                            bar_overlay.position(),
                            bar_overlay.size(),
                            Color::BLACK.scale_alpha(0.3),
                        );

                        if self.show_labels {
                            let fits = cursor_pos.y >= 10.0;
                            let label_y = if value == 0.0 {
                                bounds.height - bottom_margin - 15.0
                            } else {
                                cursor_pos.y
                            };

                            frame.fill_text(canvas::Text {
                                content: if value == 0.0 {
                                    "0".to_string()
                                } else {
                                    format!("{:.1}", value)
                                },
                                position: Point::new(cursor_pos.x, label_y),
                                color: palette.background.base.text,
                                size: Pixels(12.0),
                                font: Font::MONOSPACE,
                                align_x: Center.into(),
                                align_y: if fits { Bottom } else { Top },
                                ..canvas::Text::default()
                            });
                        }
                    }
                    _ => {}
                }
            }

            // Draw grid lines if enabled
            if self.show_grid {
                // Draw horizontal grid lines in the chart area only
                let grid_steps = 5;
                for i in 0..=grid_steps {
                    let y = (bounds.height - bottom_margin) * (i as f32 / grid_steps as f32);
                    frame.fill_rectangle(
                        Point::new(0.0, y),
                        Size::new(bounds.width, 1.0),
                        palette.background.base.text.scale_alpha(0.1),
                    );
                    
                    // Add value labels on the left
                    if self.show_labels {
                        let grid_value = max_value * (1.0 - i as f64 / grid_steps as f64);
                        frame.fill_text(canvas::Text {
                            content: format!("{:.0}", grid_value),
                            position: Point::new(5.0, y - 2.0),
                            color: palette.background.base.text.scale_alpha(0.6),
                            size: Pixels(10.0),
                            font: Font::MONOSPACE,
                            align_y: Bottom,
                            ..canvas::Text::default()
                        });
                    }
                }

                // Draw a baseline at zero
                let zero_y = bounds.height - bottom_margin;
                frame.fill_rectangle(
                    Point::new(0.0, zero_y),
                    Size::new(bounds.width, 2.0),
                    palette.background.base.text.scale_alpha(0.3),
                );

                // Draw vertical grid lines
                let vertical_steps = (visible_bars / 2).max(1).min(10);
                for i in 0..=vertical_steps {
                    let x = bounds.width * (i as f32 / vertical_steps as f32);
                    frame.fill_rectangle(
                        Point::new(x, 0.0),
                        Size::new(1.0, bounds.height - bottom_margin),
                        palette.background.base.text.scale_alpha(0.1),
                    );
                }
            }

            // Draw average line if labels are enabled
            if self.show_labels && average > 0.0 {
                let average_y = bounds.height - bottom_margin - (average * pixels_per_unit as f64) as f32;
                frame.fill_rectangle(
                    Point::new(0.0, average_y),
                    Size::new(bounds.width, 2.0),
                    Color::from_rgb(0.0, 0.6, 1.0).scale_alpha(0.7),
                );

                frame.fill_text(canvas::Text {
                    content: format!("Avg: {:.1}", average),
                    position: Point::new(bounds.width - 5.0, average_y - 2.0),
                    color: Color::from_rgb(0.0, 0.6, 1.0),
                    size: Pixels(12.0),
                    font: Font::MONOSPACE,
                    align_x: Right.into(),
                    align_y: Bottom,
                    ..canvas::Text::default()
                });
            }
        });

        vec![geometry]
    }
}
