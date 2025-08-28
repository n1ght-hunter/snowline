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
    Function(Box<dyn Fn(&BarColorParams) -> Color + Send + Sync>),
}

impl BarColorScheme {
    /// Call the color scheme to get a color for the given parameters
    pub fn call(&self, params: &BarColorParams) -> Color {
        match self {
            BarColorScheme::Single(color) => *color,
            BarColorScheme::Function(function) => function(params),
        }
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
            BarColorScheme::Function(_) => {
                // Can't clone functions, so return default
                Self::default()
            }
        }
    }
}
