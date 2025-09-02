//! Shared drawing utilities for all graph types

use iced::{Color, Font, Pixels, Point, Size, Theme, widget::canvas};

/// Common grid drawing functionality
pub struct GridConfig {
    pub padding: f32,
    pub chart_width: f32,
    pub chart_height: f32,
    pub horizontal_lines: usize,
    pub vertical_lines: usize,
    pub major_alpha: f32,
    pub minor_alpha: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            padding: 40.0,
            chart_width: 0.0,
            chart_height: 0.0,
            horizontal_lines: 10,
            vertical_lines: 10,
            major_alpha: 0.15,
            minor_alpha: 0.05,
        }
    }
}

/// Draw a standard grid with major and minor lines
pub fn draw_grid(
    frame: &mut canvas::Frame,
    config: &GridConfig,
    palette: &iced::theme::palette::Extended,
) {
    let major_grid_color = palette.background.base.text.scale_alpha(config.major_alpha);
    let minor_grid_color = palette.background.base.text.scale_alpha(config.minor_alpha);

    // Draw horizontal grid lines
    for i in 0..=config.horizontal_lines {
        let y = config.padding + (i as f32 / config.horizontal_lines as f32) * config.chart_height;
        let color = if i % 2 == 0 {
            major_grid_color
        } else {
            minor_grid_color
        };
        let width = if i % 2 == 0 { 0.8 } else { 0.4 };

        frame.stroke(
            &canvas::Path::line(
                Point::new(config.padding, y),
                Point::new(config.padding + config.chart_width, y),
            ),
            canvas::Stroke::default()
                .with_color(color)
                .with_width(width),
        );
    }

    // Draw vertical grid lines
    for i in 0..=config.vertical_lines {
        let x = config.padding + (i as f32 / config.vertical_lines as f32) * config.chart_width;
        let color = if i % 2 == 0 {
            major_grid_color
        } else {
            minor_grid_color
        };
        let width = if i % 2 == 0 { 0.8 } else { 0.4 };

        frame.stroke(
            &canvas::Path::line(
                Point::new(x, config.padding),
                Point::new(x, config.padding + config.chart_height),
            ),
            canvas::Stroke::default()
                .with_color(color)
                .with_width(width),
        );
    }

    // Draw chart border
    let border_color = palette.background.base.text.scale_alpha(0.3);
    frame.stroke(
        &canvas::Path::rectangle(
            Point::new(config.padding, config.padding),
            Size::new(config.chart_width, config.chart_height),
        ),
        canvas::Stroke::default()
            .with_color(border_color)
            .with_width(2.0),
    );
}

/// Draw value labels on the Y-axis
#[allow(clippy::too_many_arguments)]
pub fn draw_y_axis_labels(
    frame: &mut canvas::Frame,
    padding: f32,
    chart_height: f32,
    min_value: f64,
    max_value: f64,
    steps: usize,
    theme: &Theme,
    unit_suffix: &str,
) {
    let palette = theme.extended_palette();
    let text_color = palette.background.base.text;
    let value_range = max_value - min_value;

    for i in 0..=steps {
        let y = padding + (i as f32 / steps as f32) * chart_height;
        let value = max_value - (i as f64 / steps as f64) * value_range;

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
            content: if unit_suffix.is_empty() {
                format!("{:.0}", value)
            } else {
                format!("{:.0}{}", value, unit_suffix)
            },
            position: Point::new(padding - 5.0, y),
            color: text_color,
            size: Pixels(9.0),
            font: Font::MONOSPACE,
            align_x: iced::alignment::Horizontal::Right.into(),
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }
}

/// Draw an average line across the chart
#[allow(clippy::too_many_arguments)]
pub fn draw_average_line(
    frame: &mut canvas::Frame,
    padding: f32,
    chart_width: f32,
    chart_height: f32,
    average: f64,
    min_value: f64,
    max_value: f64,
    color: Color,
) {
    let value_range = max_value - min_value;
    if value_range == 0.0 {
        return;
    }

    let normalized_avg = (average - min_value) / value_range;
    let avg_y = padding + chart_height - (normalized_avg as f32 * chart_height);

    // Draw dashed line
    let segment_length = 15.0;
    let gap_length = 5.0;
    let total_width = chart_width;
    let num_segments = (total_width / (segment_length + gap_length)) as usize;

    // Glow effect behind the average line
    for i in 0..num_segments {
        let start_x = padding + (i as f32) * (segment_length + gap_length);
        let end_x = (start_x + segment_length).min(padding + chart_width);

        // Glow effect
        frame.stroke(
            &canvas::Path::line(Point::new(start_x, avg_y), Point::new(end_x, avg_y)),
            canvas::Stroke::default()
                .with_color(color.scale_alpha(0.3))
                .with_width(6.0),
        );
    }

    // Main average line segments
    for i in 0..num_segments {
        let start_x = padding + (i as f32) * (segment_length + gap_length);
        let end_x = (start_x + segment_length).min(padding + chart_width);

        frame.stroke(
            &canvas::Path::line(Point::new(start_x, avg_y), Point::new(end_x, avg_y)),
            canvas::Stroke::default().with_color(color).with_width(3.0),
        );
    }
}
