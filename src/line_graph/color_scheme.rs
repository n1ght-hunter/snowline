use iced::{Color, Theme};

/// Parameters passed to point color functions
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PointColorParams<'a> {
    /// The index/position of the point (0-based)
    pub index: usize,
    /// The value of the point
    pub value: f64,
    /// The average value across all points
    pub average: f64,
    /// The current theme for theme-aware coloring
    pub theme: &'a Theme,
}

pub enum PointColorScheme {
    Single(Color),
    Function(Box<dyn Fn(&PointColorParams) -> Color + Send + Sync>),
}

impl PointColorScheme {
    /// Call the color scheme to get a color for the given parameters
    pub fn call(&self, params: &PointColorParams) -> Color {
        match self {
            PointColorScheme::Single(color) => *color,
            PointColorScheme::Function(function) => function(params),
        }
    }

    /// Create a new function-based color scheme
    pub fn new_function<F>(function: F) -> Self
    where
        F: Fn(&PointColorParams) -> Color + Send + Sync + 'static,
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

    /// Gradient color scheme from green to red
    pub fn gradient() -> Self {
        Self::new_function(|params| {
            let ratio = (params.value / params.average).min(2.0).max(0.5);
            let normalized = ((ratio - 1.0) / 1.0).max(-1.0).min(1.0);

            if normalized <= 0.0 {
                let t = (-normalized) as f32;
                Color::from_rgb(0.2 + t * 0.8, 0.8, 0.2)
            } else {
                let t = normalized as f32;
                Color::from_rgb(1.0, 0.8 - t * 0.6, 0.2)
            }
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

impl Default for PointColorScheme {
    fn default() -> Self {
        Self::performance()
    }
}

impl Clone for PointColorScheme {
    fn clone(&self) -> Self {
        match self {
            PointColorScheme::Single(color) => PointColorScheme::Single(*color),
            PointColorScheme::Function(_) => {
                // Can't clone functions, so return default
                Self::default()
            }
        }
    }
}
