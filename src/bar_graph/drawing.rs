//! Drawing utilities for bar graphs

use super::{BarGraph, color_scheme::BarColorParams};
use crate::utils::ValueMapper;
use iced::{
    Bottom, Center, Color, Font, Pixels, Point, Rectangle, Right, Size, Theme, Top, widget::canvas,
};

impl<'a, I, T, M> BarGraph<'a, I, T, M>
where
    I: Iterator<Item = T> + Clone + 'a,
    M: ValueMapper<T>,
{
    /// Draw the bars themselves
    pub(super) fn draw_bars(
        &self,
        frame: &mut canvas::Frame,
        bounds: Size,
        visible_bars: usize,
        values: &[f64],
        average: f64,
        max_value: f64,
        theme: &Theme,
    ) {
        if values.is_empty() || max_value == 0.0 {
            return;
        }

        let bar_width = bounds.width / visible_bars as f32;
        let bottom_margin = 40.0;
        let available_height = bounds.height - bottom_margin;
        let pixels_per_unit = available_height / max_value as f32;

        for (i, value) in values.iter().take(visible_bars).enumerate() {
            let value = *value;

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
            let bar_color = {
                let params = BarColorParams {
                    index: i,
                    value,
                    average,
                    theme,
                };

                // For zero values, use a special muted color unless overridden
                if value == 0.0 {
                    // Let the function decide, but provide a fallback
                    let function_color = self.bar_color_scheme.call(&params);
                    // If it's the default performance color, use muted instead
                    if function_color == Color::from_rgb(0.2, 0.8, 0.3)
                        || function_color == Color::from_rgb(0.9, 0.3, 0.3)
                        || function_color == Color::from_rgb(1.0, 0.7, 0.2)
                    {
                        Color::from_rgb(0.7, 0.7, 0.7)
                    } else {
                        function_color
                    }
                } else {
                    self.bar_color_scheme.call(&params)
                }
            };

            frame.fill_rectangle(bar.position(), bar.size(), self.bar_color.unwrap_or(bar_color));
        }
    }

    /// Draw bar labels and hover effects
    pub(super) fn draw_bar_labels_and_hover(
        &self,
        frame: &mut canvas::Frame,
        bounds: Size,
        visible_bars: usize,
        values: &[f64],
        cursor: Option<Point>,
        theme: &Theme,
    ) {
        if values.is_empty() {
            return;
        }

        let bar_width = bounds.width / visible_bars as f32;
        let bottom_margin = 40.0;
        let palette = theme.extended_palette();

        for (i, value) in values.iter().take(visible_bars).enumerate() {
            let value = *value;

            // Draw bar index labels at bottom
            if self.show_labels {
                frame.fill_text(canvas::Text {
                    content: format!("{}", i),
                    position: Point::new(
                        i as f32 * bar_width + bar_width / 2.0,
                        bounds.height - 5.0,
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

            if let Some(cursor_pos) = cursor {
                if bar_overlay.contains(cursor_pos) {
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
            }
        }
    }

    /// Draw grid lines and scale labels
    pub(super) fn draw_grid_and_scale(
        &self,
        frame: &mut canvas::Frame,
        bounds: Size,
        visible_bars: usize,
        max_value: f64,
        theme: &Theme,
    ) {
        if !self.show_grid {
            return;
        }

        let palette = theme.extended_palette();
        let bottom_margin = 40.0;

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

    /// Draw average line and label
    pub(super) fn draw_average_line(
        &self,
        frame: &mut canvas::Frame,
        bounds: Size,
        average: f64,
        max_value: f64,
    ) {
        if !self.show_labels || average <= 0.0 || max_value == 0.0 {
            return;
        }

        let bottom_margin = 40.0;
        let available_height = bounds.height - bottom_margin;
        let pixels_per_unit = available_height / max_value as f32;
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
}
