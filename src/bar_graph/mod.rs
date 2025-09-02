pub use canvas::Cache;
use iced::{Color, Event, Rectangle, Renderer, Theme, mouse, widget::canvas};

// Make modules public for prelude access, but don't re-export types here
pub mod color_scheme;
use color_scheme::{BarColorParams, BarColorScheme};

pub mod state;
use state::BarGraphState;

// Import drawing utilities
mod drawing;

// Re-export the shared interaction type for backward compatibility
pub use crate::utils::BarInteraction as Interaction;
use crate::utils::{DefaultMap, ValueMapper};

#[derive(Debug, Clone, Copy)]
pub enum BinAggregator {
    Average,
    Sum,
    Max,
}

#[allow(missing_debug_implementations)]
pub struct BarGraph<'a, I, T, M = DefaultMap>
where
    I: Iterator<Item = T> + Clone + 'a,
{
    pub datapoints: I,
    pub cache: &'a canvas::Cache,
    pub bar_color: Option<Color>,
    pub bar_width: f32,
    pub show_grid: bool,
    pub show_labels: bool,
    pub base_bars: f32, // Target number of bars (bins)
    pub bar_color_scheme: BarColorScheme,
    pub mapper: M,
    pub bin_aggregator: BinAggregator,
}

impl<'a, I, T, M> BarGraph<'a, I, T, M>
where
    I: Iterator<Item = T> + Clone + 'a,
    M: ValueMapper<T>,
{
    pub fn with_mapper(datapoints: I, cache: &'a canvas::Cache, mapper: M) -> Self {
        Self {
            datapoints,
            cache,
            bar_color: None,
            bar_width: 2.0,
            show_grid: true,
            show_labels: true,
            base_bars: 50.0,
            bar_color_scheme: BarColorScheme::default(),
            mapper,
            bin_aggregator: BinAggregator::Average,
        }
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

    /// Set exact number of bins (bars)
    pub fn bins(mut self, count: usize) -> Self {
        self.base_bars = (count as f32).max(1.0);
        self
    }

    /// Choose how to aggregate values inside each bin
    pub fn bin_aggregator(mut self, kind: BinAggregator) -> Self {
        self.bin_aggregator = kind;
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

    fn desired_bins(&self, total_items: usize) -> usize {
        let desired = self.base_bars.max(1.0) as usize;
        desired.min(total_items.max(1))
    }
}

// Default constructor using Into<f64> without custom mapper
impl<'a, I, T> BarGraph<'a, I, T, DefaultMap>
where
    I: Iterator<Item = T> + Clone + 'a,
    T: Copy + Into<f64>,
{
    pub fn new(datapoints: I, cache: &'a canvas::Cache) -> Self {
        Self {
            datapoints,
            cache,
            bar_color: None,
            bar_width: 2.0,
            show_grid: true,
            show_labels: true,
            base_bars: 50.0,
            bar_color_scheme: BarColorScheme::default(),
            mapper: DefaultMap,
            bin_aggregator: BinAggregator::Average,
        }
    }
}

impl<'a, I, T, M> canvas::Program<Interaction> for BarGraph<'a, I, T, M>
where
    I: Iterator<Item = T> + Clone + 'a,
    M: ValueMapper<T>,
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
                    let total_items = self.datapoints.clone().count();
                    let visible_bars = self.desired_bins(total_items);

                    if visible_bars > 0 {
                        let bar_width = bounds.width / visible_bars as f32;
                        let bar_index = (cursor_position.x / bar_width) as usize;

                        if bar_index < visible_bars {
                            if state.hovered_bar != Some(bar_index) {
                                state.hovered_bar = Some(bar_index);
                                self.cache.clear();
                                return Some(canvas::Action::publish(Interaction::BarHovered(
                                    bar_index,
                                )));
                            }
                        } else if state.hovered_bar.is_some() {
                            state.hovered_bar = None;
                            self.cache.clear();
                            return Some(canvas::Action::request_redraw());
                        }
                    }
                } else if state.hovered_bar.is_some() {
                    state.hovered_bar = None;
                    self.cache.clear();
                    return Some(canvas::Action::request_redraw());
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
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let cursor = cursor.position_in(bounds);
            let bounds = frame.size();

            // Collect raw values via the mapper
            let values_all: Vec<f64> = self
                .datapoints
                .clone()
                .map(|v| self.mapper.map(&v))
                .collect();

            if values_all.is_empty() {
                return;
            }

            // Aggregate into bins
            let desired_bins = self.desired_bins(values_all.len());
            let bin_size = ((values_all.len() as f32) / desired_bins as f32)
                .ceil()
                .max(1.0) as usize;
            let mut binned: Vec<f64> = Vec::with_capacity(desired_bins);
            let mut i = 0;
            while i < values_all.len() {
                let end = (i + bin_size).min(values_all.len());
                let slice = &values_all[i..end];
                let value = match self.bin_aggregator {
                    BinAggregator::Average => {
                        let sum: f64 = slice.iter().sum();
                        sum / slice.len() as f64
                    }
                    BinAggregator::Sum => slice.iter().sum(),
                    BinAggregator::Max => slice
                        .iter()
                        .cloned()
                        .fold(f64::NEG_INFINITY, |a, b| a.max(b)),
                };
                binned.push(value);
                i = end;
            }

            let visible_bars = binned.len();
            let average = binned.iter().copied().sum::<f64>() / visible_bars as f64;
            let max_value = binned.iter().fold(0.0f64, |a, &b| a.max(b));
            if max_value == 0.0 {
                return;
            }

            // Draw all components using the modular functions
            self.draw_bars(
                frame,
                bounds,
                visible_bars,
                &binned,
                average,
                max_value,
                theme,
            );
            self.draw_grid_and_scale(frame, bounds, visible_bars, max_value, theme);
            self.draw_average_line(frame, bounds, average, max_value);
            self.draw_bar_labels_and_hover(frame, bounds, visible_bars, &binned, cursor, theme);
        });

        vec![geometry]
    }
}
