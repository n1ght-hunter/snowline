use iced::{Color, Theme};

/// Parameters passed to bar color functions
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BarColorParams<'a> {
    /// The index/position of the bar (0-based)
    pub index: usize,
    /// The value of the bar
    pub value: f64,
    /// The average value across all bars
    pub average: f64,
    /// The current theme for theme-aware coloring
    pub theme: &'a Theme,
}

pub enum BarColorScheme {
    Single(Color),
    Palette(Vec<Color>),
    Function(Box<dyn Fn(&BarColorParams) -> Color + Send + Sync>),
}

impl BarColorScheme {
    /// Call the color scheme to get a color for the given parameters
    /// The returned color is adjusted based on value vs. average so all
    /// bars respond to data magnitude consistently.
    pub fn call(&self, params: &BarColorParams) -> Color {
        let base = match self {
            BarColorScheme::Single(color) => *color,
            BarColorScheme::Palette(colors) => {
                if colors.is_empty() {
                    Color::from_rgb(0.7, 0.7, 0.7)
                } else {
                    let idx = params.index % colors.len();
                    colors[idx]
                }
            }
            BarColorScheme::Function(function) => function(params),
        };

        adjust_color_by_deviation(base, params)
    }

    /// Create a new function-based color scheme
    pub fn new_function<F>(function: F) -> Self
    where
        F: Fn(&BarColorParams) -> Color + Send + Sync + 'static,
    {
        Self::Function(Box::new(function))
    }

    /// Create a single color scheme
    pub fn new_single(color: Color) -> Self {
        Self::Single(color)
    }

    /// Create a palette-based color scheme that cycles colors by index
    pub fn palette(colors: Vec<Color>) -> Self {
        Self::Palette(colors)
    }

    /// Default performance-based color scheme
    pub fn performance() -> Self {
        Self::new_function(|params| {
            if params.value < params.average * 0.7 {
                Color::from_rgb(0.2, 0.8, 0.3) // Green for good performance
            } else if params.value > params.average * 1.3 {
                Color::from_rgb(0.9, 0.3, 0.3) // Red for poor performance
            } else {
                Color::from_rgb(1.0, 0.7, 0.2) // Orange for average performance
            }
        })
    }

    /// Theme-aware color scheme
    pub fn theme_colors() -> Self {
        Self::new_function(|params| {
            let palette = params.theme.extended_palette();
            palette.primary.base.color
        })
    }

    /// Index-based traffic light colors
    pub fn traffic_light() -> Self {
        Self::new_function(|params| {
            if params.index < 6 {
                Color::from_rgb(0.2, 0.8, 0.3) // Green
            } else if params.index == 6 {
                Color::from_rgb(1.0, 0.9, 0.0) // Yellow
            } else {
                Color::from_rgb(0.9, 0.3, 0.3) // Red
            }
        })
    }
}

impl Default for BarColorScheme {
    fn default() -> Self {
        Self::performance()
    }
}

impl Clone for BarColorScheme {
    fn clone(&self) -> Self {
        match self {
            BarColorScheme::Single(color) => BarColorScheme::Single(*color),
            BarColorScheme::Palette(colors) => BarColorScheme::Palette(colors.clone()),
            BarColorScheme::Function(_) => {
                // Can't clone functions, so return default
                Self::default()
            }
        }
    }
}

// --- helpers ---

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix_color(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgba(
        mix(a.r, b.r, t),
        mix(a.g, b.g, t),
        mix(a.b, b.b, t),
        mix(a.a, b.a, t),
    )
}

/// Adjust a base color toward green when below average and toward red
/// when above average. The strength is proportional to deviation.
fn adjust_color_by_deviation(base: Color, params: &BarColorParams) -> Color {
    if params.average <= 0.0 || !params.value.is_finite() || !params.average.is_finite() {
        return base;
    }

    let ratio = (params.value / params.average).clamp(0.5, 2.0);
    let normalized = (ratio - 1.0).clamp(-1.0, 1.0) as f32; // [-1.0, 1.0]

    // Targets: green for below average, red for above average
    let below_target = Color::from_rgb(0.2, 0.8, 0.3);
    let above_target = Color::from_rgb(0.9, 0.3, 0.3);

    if normalized < 0.0 {
        let t = -normalized; // 0..1
        mix_color(base, below_target, t)
    } else if normalized > 0.0 {
        let t = normalized; // 0..1
        mix_color(base, above_target, t)
    } else {
        base
    }
}
