pub use canvas::Cache;
use iced::{
    Center, Color, Event, Font, Pixels, Point, Rectangle, Renderer, Right, Size, Theme, mouse,
    widget::canvas,
};

use crate::{
    utils::{DefaultMap, GridConfig, ValueMapper, draw_grid},
    zoom::Zoom,
};

// Make modules public for prelude access, but don't re-export types here
pub mod color_scheme;
use color_scheme::{PointColorParams, PointColorScheme};

pub mod state;
use state::LineGraphState;

// Re-export the shared interaction type for backward compatibility
pub use crate::utils::LineInteraction as Interaction;

#[allow(missing_debug_implementations)]
pub struct LineGraph<'a, I, T, M = DefaultMap>
where
    I: Iterator<Item = T> + Clone + 'a,
{
    pub datapoints: I,
    pub cache: &'a canvas::Cache,
    pub line_color: Option<Color>,
    pub line_width: f32,
    pub show_points: bool,
    pub point_radius: f32,
    pub show_grid: bool,
    pub show_labels: bool,
    pub zoom: Zoom,
    pub base_points: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub point_color_scheme: PointColorScheme,
    pub mapper: M,
    pub external_zoom: Option<Zoom>, // Optional external zoom override
}

impl<'a, I, T> LineGraph<'a, I, T>
where
    I: Iterator<Item = T> + Clone + 'a,
    T: Copy + Into<f64>,
{
    pub fn new(datapoints: I, cache: &'a canvas::Cache) -> Self {
        Self {
            datapoints,
            cache,
            line_color: None,
            line_width: 2.0,
            show_points: true,
            point_radius: 3.0,
            show_grid: true,
            show_labels: true,
            zoom: Zoom::default(),
            base_points: 50.0, // Increased default from 20.0
            zoom_min: 0.1,
            zoom_max: 10.0,
            point_color_scheme: PointColorScheme::default(),
            mapper: DefaultMap,
            external_zoom: None,
        }
    }
}

impl<'a, I, T, M> LineGraph<'a, I, T, M>
where
    I: Iterator<Item = T> + Clone + 'a,
    M: ValueMapper<T>,
{
    /// Construct with a custom mapper implementation
    pub fn with_mapper(datapoints: I, cache: &'a canvas::Cache, mapper: M) -> Self {
        Self {
            datapoints,
            cache,
            line_color: None,
            line_width: 2.0,
            show_points: true,
            point_radius: 3.0,
            show_grid: true,
            show_labels: true,
            zoom: Zoom::default(),
            base_points: 50.0,
            zoom_min: 0.1,
            zoom_max: 10.0,
            point_color_scheme: PointColorScheme::default(),
            mapper,
            external_zoom: None,
        }
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

    pub fn line_color(mut self, color: Color) -> Self {
        self.line_color = Some(color);
        self
    }

    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    pub fn show_points(mut self, show: bool) -> Self {
        self.show_points = show;
        self
    }

    pub fn point_radius(mut self, radius: f32) -> Self {
        self.point_radius = radius;
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

    pub fn base_points(mut self, points: f32) -> Self {
        self.base_points = points;
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

    pub fn point_color_fn<F>(mut self, color_fn: F) -> Self
    where
        F: Fn(&PointColorParams) -> Color + Send + Sync + 'static,
    {
        self.point_color_scheme = PointColorScheme::new_function(color_fn);
        self
    }

    pub fn single_point_color(mut self, color: Color) -> Self {
        self.point_color_scheme = PointColorScheme::new_single(color);
        self
    }

    pub fn point_color_scheme(mut self, scheme: PointColorScheme) -> Self {
        self.point_color_scheme = scheme;
        self
    }

    /// Default performance-based color scheme (green for good, red for poor, orange for average)
    pub fn performance_colors(mut self) -> Self {
        self.point_color_scheme = PointColorScheme::performance();
        self
    }

    /// Theme-aware color scheme that adapts to the current theme
    pub fn theme_colors(mut self) -> Self {
        self.point_color_scheme = PointColorScheme::theme_colors();
        self
    }

    /// Gradient color scheme from green to red based on value relative to average
    pub fn gradient_colors(mut self) -> Self {
        self.point_color_scheme = PointColorScheme::gradient();
        self
    }

    /// Index-based color scheme: green for index < 6, yellow for index 6, red for index > 6
    pub fn traffic_light_colors(mut self) -> Self {
        self.point_color_scheme = PointColorScheme::traffic_light();
        self
    }
}

// Helper methods for LineGraph
impl<'a, I, T, M> LineGraph<'a, I, T, M>
where
    I: Iterator<Item = T> + Clone + 'a,
    M: ValueMapper<T>,
{
    /// Get the effective zoom value (external zoom if set, otherwise state zoom)
    fn effective_zoom(&self, state: &LineGraphState) -> Zoom {
        self.external_zoom.unwrap_or(state.zoom)
    }
}

impl<'a, I, T, M> canvas::Program<Interaction> for LineGraph<'a, I, T, M>
where
    I: Iterator<Item = T> + Clone + 'a,
    M: ValueMapper<T>,
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
                // Only enable hover when zoomed in (not in full view) and points are visible
                let effective_zoom = self.effective_zoom(state);
                if effective_zoom.is_value() {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        let new_hovered = self.find_nearest_point(cursor_position, bounds, state);

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
                if let Some(cursor_position) = cursor.position_in(bounds)
                    && let Some(point_index) =
                        self.find_nearest_point(cursor_position, bounds, state)
                {
                    return Some(canvas::Action::publish(Interaction::PointClicked(
                        point_index,
                    )));
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
                                // Zooming in
                                state.zoom.increment_with_limits(self.zoom_max)
                            } else {
                                // Zooming out
                                state.zoom.decrement_with_limits(self.zoom_min)
                            };

                            if new_zoom != state.zoom {
                                state.zoom = new_zoom;
                                self.cache.clear();
                                return Some(
                                    canvas::Action::publish(Interaction::ZoomChanged(new_zoom))
                                        .and_capture(),
                                );
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
            let base_points = self.base_points; // Use configurable base points
            let max_visible_points = match effective_zoom {
                Zoom::Full => {
                    // Full view mode - display all available data points
                    all_datapoints.len()
                }
                Zoom::Value(zoom_factor) => {
                    if zoom_factor >= 1.0 {
                        // Zooming in: show fewer points for more detail
                        (base_points / zoom_factor).max(5.0) as usize
                    } else {
                        // Zooming out: show more points for broader view
                        (base_points / zoom_factor).min(all_datapoints.len() as f32) as usize
                    }
                }
            };

            let start_index = match effective_zoom {
                Zoom::Full => {
                    // In full view mode, show all data from the beginning
                    0
                }
                Zoom::Value(_) => {
                    if all_datapoints.len() > max_visible_points {
                        // For other zoom levels, show the most recent data
                        all_datapoints.len() - max_visible_points
                    } else {
                        0
                    }
                }
            };

            let visible_datapoints: Vec<_> = all_datapoints.into_iter().skip(start_index).collect();

            if visible_datapoints.is_empty() {
                return;
            }

            // Find min/max values for proper scaling from visible data
            let values: Vec<f64> = visible_datapoints
                .iter()
                .map(|(_, v)| self.mapper.map(v))
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
                    let value_f64 = self.mapper.map(value);
                    let normalized_value = (value_f64 - min_value) / value_range;
                    let y = padding + chart_height - (normalized_value as f32 * chart_height);
                    Point::new(x, y)
                })
                .collect();

            // Draw grid if enabled
            if self.show_grid {
                self.draw_grid(frame, padding, chart_width, chart_height, palette);
            }

            // Draw the line
            if points.len() > 1 {
                self.draw_line(frame, &points, palette);
            }

            // Draw data points if enabled (but not in full view)
            if self.show_points && effective_zoom.is_value() {
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
                    .map(|(i, v)| (*i, self.mapper.map(v)))
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
                    palette,
                    effective_zoom,
                    &visible_datapoints_f64,
                );
            }
        });

        vec![geometry]
    }
}

impl<'a, I, T, M> LineGraph<'a, I, T, M>
where
    I: Iterator<Item = T> + Clone + 'a,
    M: ValueMapper<T>,
{
    fn find_nearest_point(
        &self,
        cursor_pos: Point,
        bounds: Rectangle,
        state: &LineGraphState,
    ) -> Option<usize> {
        let padding = 40.0;
        let chart_width = bounds.width - 2.0 * padding;

        // We need to use the same logic as the draw method to calculate visible datapoints
        let all_datapoints: Vec<_> = self.datapoints.clone().enumerate().collect();

        if all_datapoints.is_empty() {
            return None;
        }

        // Use the same zoom calculation logic as the draw method
        let effective_zoom = self.effective_zoom(state);
        let base_points = self.base_points;

        let max_visible_points = match effective_zoom {
            Zoom::Full => {
                // Full view mode - display all available data points
                all_datapoints.len()
            }
            Zoom::Value(zoom_factor) => {
                if zoom_factor >= 1.0 {
                    (base_points / zoom_factor).max(5.0) as usize
                } else {
                    (base_points / zoom_factor).min(all_datapoints.len() as f32) as usize
                }
            }
        };

        let start_index = match effective_zoom {
            Zoom::Full => {
                // In full view mode, show all data from the beginning
                0
            }
            Zoom::Value(_) => {
                if all_datapoints.len() > max_visible_points {
                    all_datapoints.len() - max_visible_points
                } else {
                    0
                }
            }
        };

        let visible_datapoints: Vec<_> = all_datapoints.into_iter().skip(start_index).collect();

        if visible_datapoints.is_empty() {
            return None;
        }

        // Find the nearest point using the same coordinate calculation as drawing
        let mut closest_index = None;
        let mut closest_distance = f32::INFINITY;

        // We also need the y-coordinates to calculate true distance
        let values: Vec<f64> = visible_datapoints
            .iter()
            .map(|(_, v)| self.mapper.map(v))
            .collect();

        if values.is_empty() {
            return None;
        }

        let min_value = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_value = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let value_range = max_value - min_value;

        if value_range == 0.0 {
            return None;
        }

        let chart_height = bounds.height - 2.0 * padding;

        for (i, (_original_index, _)) in visible_datapoints.iter().enumerate() {
            let x =
                padding + (i as f32 / (visible_datapoints.len() - 1).max(1) as f32) * chart_width;

            // Calculate y coordinate using same logic as drawing
            let value_f64 = values[i];
            let normalized_value = (value_f64 - min_value) / value_range;
            let y = padding + chart_height - (normalized_value as f32 * chart_height);

            // Calculate distance to the actual point (both x and y)
            let dx = cursor_pos.x - x;
            let dy = cursor_pos.y - y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Use a slightly larger radius for better usability
            if distance <= 20.0 && distance < closest_distance {
                closest_distance = distance;
                closest_index = Some(i);
            }
        }

        closest_index
    }

    fn draw_grid(
        &self,
        frame: &mut canvas::Frame,
        padding: f32,
        chart_width: f32,
        chart_height: f32,
        palette: &iced::theme::palette::Extended,
    ) {
        let config = GridConfig {
            padding,
            chart_width,
            chart_height,
            horizontal_lines: 10,
            vertical_lines: 10,
            ..GridConfig::default()
        };
        draw_grid(frame, &config, palette);
    }

    fn draw_line(
        &self,
        frame: &mut canvas::Frame,
        points: &[Point],
        _palette: &iced::theme::palette::Extended,
    ) {
        if points.len() < 2 {
            return;
        }

        // Draw shadow/glow effect behind the main line
        let mut shadow_path_builder = canvas::path::Builder::new();
        shadow_path_builder.move_to(Point::new(points[0].x + 1.0, points[0].y + 1.0));

        for point in points.iter().skip(1) {
            shadow_path_builder.line_to(Point::new(point.x + 1.0, point.y + 1.0));
        }

        let shadow_path = shadow_path_builder.build();
        frame.stroke(
            &shadow_path,
            canvas::Stroke::default()
                .with_color(Color::BLACK.scale_alpha(0.2))
                .with_width(self.line_width + 1.0),
        );

        // Draw main line with smooth appearance
        let mut path_builder = canvas::path::Builder::new();
        path_builder.move_to(points[0]);

        for point in points.iter().skip(1) {
            path_builder.line_to(*point);
        }

        let path = path_builder.build();
        let line_color = self.line_color.unwrap_or(Color::from_rgb(0.2, 0.6, 1.0)); // Nice blue

        frame.stroke(
            &path,
            canvas::Stroke::default()
                .with_color(line_color)
                .with_width(self.line_width),
        );

        // Add a subtle glow effect
        frame.stroke(
            &path,
            canvas::Stroke::default()
                .with_color(line_color.scale_alpha(0.3))
                .with_width(self.line_width + 2.0),
        );
    }

    fn draw_points(
        &self,
        frame: &mut canvas::Frame,
        points: &[Point],
        values: &[f64],
        state: &Option<usize>,
        average: f64,
        theme: &Theme,
    ) {
        for (i, (point, value)) in points.iter().zip(values.iter()).enumerate() {
            let is_hovered = *state == Some(i);

            // Use the point color scheme to determine color
            let point_color = {
                let params = PointColorParams {
                    index: i,
                    value: *value,
                    average,
                    theme,
                };
                self.point_color_scheme.call(&params)
            };

            let base_radius = self.point_radius;
            let radius = if is_hovered {
                base_radius + 3.0
            } else {
                base_radius
            };

            // Draw point shadow
            frame.fill(
                &canvas::Path::circle(Point::new(point.x + 1.0, point.y + 1.0), radius),
                Color::BLACK.scale_alpha(0.3),
            );

            // Draw outer ring for depth
            frame.fill(
                &canvas::Path::circle(*point, radius + 1.0),
                Color::WHITE.scale_alpha(0.8),
            );

            // Draw main point
            frame.fill(&canvas::Path::circle(*point, radius), point_color);

            // Add highlight to make it look more 3D
            frame.fill(
                &canvas::Path::circle(
                    Point::new(point.x - radius * 0.3, point.y - radius * 0.3),
                    radius * 0.4,
                ),
                Color::WHITE.scale_alpha(0.6),
            );

            // Show enhanced tooltip on hover
            if is_hovered {
                let tooltip_bg = Color::from_rgba(0.1, 0.1, 0.1, 0.9);
                let tooltip_text = Color::WHITE;
                let tooltip_width = 80.0;
                let tooltip_height = 25.0;
                let tooltip_x = point.x - tooltip_width / 2.0;
                let tooltip_y = point.y - radius - tooltip_height - 8.0;

                // Tooltip background with rounded corners effect
                frame.fill(
                    &canvas::Path::rectangle(
                        Point::new(tooltip_x, tooltip_y),
                        Size::new(tooltip_width, tooltip_height),
                    ),
                    tooltip_bg,
                );

                // Tooltip border
                frame.stroke(
                    &canvas::Path::rectangle(
                        Point::new(tooltip_x, tooltip_y),
                        Size::new(tooltip_width, tooltip_height),
                    ),
                    canvas::Stroke::default()
                        .with_color(point_color)
                        .with_width(1.5),
                );

                frame.fill_text(canvas::Text {
                    content: format!("{:.2}ms", value * 1000.0), // Convert to milliseconds
                    position: Point::new(point.x, tooltip_y + tooltip_height / 2.0),
                    color: tooltip_text,
                    size: Pixels(11.0),
                    font: Font::MONOSPACE,
                    align_x: Center.into(),
                    align_y: Center.into(),
                    ..canvas::Text::default()
                });
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_labels(
        &self,
        frame: &mut canvas::Frame,
        bounds: Size,
        padding: f32,
        chart_width: f32,
        chart_height: f32,
        min_value: f64,
        max_value: f64,
        average: f64,
        value_range: f64,
        palette: &iced::theme::palette::Extended,
        zoom: Zoom,
        visible_datapoints: &[(usize, f64)],
    ) {
        let text_color = palette.background.base.text;
        let average_line_color = Color::from_rgb(1.0, 0.6, 0.2); // Orange color for average line

        // Enhanced average line with better visibility
        let normalized_avg = (average - min_value) / value_range;
        let avg_y = padding + chart_height - (normalized_avg as f32 * chart_height);

        // Draw a more prominent average line with solid segments and glow effect
        let segment_length = 15.0;
        let gap_length = 5.0;
        let total_width = chart_width;
        let num_segments = (total_width / (segment_length + gap_length)) as usize;

        // First draw a glow effect behind the average line
        for i in 0..num_segments {
            let start_x = padding + (i as f32) * (segment_length + gap_length);
            let end_x = (start_x + segment_length).min(padding + chart_width);

            // Glow effect
            frame.stroke(
                &canvas::Path::line(Point::new(start_x, avg_y), Point::new(end_x, avg_y)),
                canvas::Stroke::default()
                    .with_color(average_line_color.scale_alpha(0.3))
                    .with_width(6.0),
            );
        }

        // Then draw the main average line segments
        for i in 0..num_segments {
            let start_x = padding + (i as f32) * (segment_length + gap_length);
            let end_x = (start_x + segment_length).min(padding + chart_width);

            frame.stroke(
                &canvas::Path::line(Point::new(start_x, avg_y), Point::new(end_x, avg_y)),
                canvas::Stroke::default()
                    .with_color(average_line_color)
                    .with_width(3.0),
            );
        }

        // Enhanced average label positioned on the right but above the line
        let avg_label_x = padding + chart_width - 90.0;
        let avg_label_width = 85.0;
        let avg_label_height = 24.0;
        let avg_label_y = avg_y - avg_label_height - 8.0; // Position above the line

        // More prominent average label background with glow
        frame.fill(
            &canvas::Path::rectangle(
                Point::new(avg_label_x - 2.0, avg_label_y - 2.0),
                Size::new(avg_label_width + 4.0, avg_label_height + 4.0),
            ),
            average_line_color.scale_alpha(0.1),
        );

        frame.fill(
            &canvas::Path::rectangle(
                Point::new(avg_label_x, avg_label_y),
                Size::new(avg_label_width, avg_label_height),
            ),
            average_line_color.scale_alpha(0.2),
        );

        frame.stroke(
            &canvas::Path::rectangle(
                Point::new(avg_label_x, avg_label_y),
                Size::new(avg_label_width, avg_label_height),
            ),
            canvas::Stroke::default()
                .with_color(average_line_color)
                .with_width(2.0),
        );

        frame.fill_text(canvas::Text {
            content: format!("avg: {:.1}ms", average * 1000.0),
            position: Point::new(
                avg_label_x + avg_label_width / 2.0,
                avg_label_y + avg_label_height / 2.0,
            ),
            color: Color::WHITE,
            size: Pixels(12.0),
            font: Font::MONOSPACE,
            align_x: Center.into(),
            align_y: Center.into(),
            ..canvas::Text::default()
        });

        // Enhanced Y-axis labels with better formatting
        for i in 0..=5 {
            let y = padding + (i as f32 / 5.0) * chart_height;
            let value = max_value - (i as f64 / 5.0) * value_range;

            // Y-axis label background for better readability
            let label_bg_width = 35.0;
            let label_bg_height = 16.0;
            let label_bg_x = padding - label_bg_width - 2.0;
            let label_bg_y = y - label_bg_height / 2.0;

            frame.fill(
                &canvas::Path::rectangle(
                    Point::new(label_bg_x, label_bg_y),
                    Size::new(label_bg_width, label_bg_height),
                ),
                palette.background.base.color.scale_alpha(0.8),
            );

            frame.fill_text(canvas::Text {
                content: format!("{:.0}ms", value * 1000.0),
                position: Point::new(padding - 5.0, y),
                color: text_color,
                size: Pixels(9.0),
                font: Font::MONOSPACE,
                align_x: Right.into(),
                align_y: Center.into(),
                ..canvas::Text::default()
            });
        }

        // Enhanced title with zoom level information
        let title_bg_width = 220.0;
        let title_bg_height = 30.0;
        let title_bg_x = (bounds.width - title_bg_width) / 2.0;
        let title_bg_y = 5.0;

        // Title background
        frame.fill(
            &canvas::Path::rectangle(
                Point::new(title_bg_x, title_bg_y),
                Size::new(title_bg_width, title_bg_height),
            ),
            palette.background.strong.color.scale_alpha(0.1),
        );

        frame.fill_text(canvas::Text {
            content: match zoom {
                Zoom::Full => "Performance Timeline (Full View)".to_string(),
                Zoom::Value(zoom_factor) => {
                    if zoom_factor.fract() == 0.0 {
                        format!("Performance Timeline (Zoom: {}x)", zoom_factor as u32)
                    } else {
                        format!("Performance Timeline (Zoom: {:.2}x)", zoom_factor)
                    }
                }
            },
            position: Point::new(bounds.width / 2.0, 20.0),
            color: text_color,
            size: Pixels(16.0),
            font: Font::MONOSPACE,
            align_x: Center.into(),
            align_y: Center.into(),
            ..canvas::Text::default()
        });

        // Add subtitle for units and data range
        let data_info = match zoom {
            Zoom::Full => format!("(Showing all {} points)", visible_datapoints.len()),
            Zoom::Value(_) => format!("(Showing last {} points)", visible_datapoints.len()),
        };

        frame.fill_text(canvas::Text {
            content: data_info,
            position: Point::new(bounds.width / 2.0, 50.0),
            color: text_color.scale_alpha(0.7),
            size: Pixels(10.0),
            font: Font::MONOSPACE,
            align_x: Center.into(),
            ..canvas::Text::default()
        });
    }
}
