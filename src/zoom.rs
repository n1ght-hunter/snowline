//! Shared zoom functionality for all graph types

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zoom {
    /// Specific zoom level with a numeric value
    Value(f32),
    /// Full view showing all available data
    Full,
}

impl Zoom {
    /// Create a new zoom level with the given value
    pub fn new(value: f32) -> Self {
        Self::Value(value.max(0.1)) // Ensure minimum zoom
    }

    pub fn is_value(self) -> bool {
        matches!(self, Zoom::Value(_))
    }

    pub fn is_full(self) -> bool {
        matches!(self, Zoom::Full)
    }

    /// Increment zoom with default maximum of 10.0
    pub fn increment(self) -> Self {
        self.increment_with_limits(10.0)
    }

    /// Decrement zoom with default minimum of 0.1
    pub fn decrement(self) -> Self {
        self.decrement_with_limits(0.1)
    }

    /// Increment zoom with a specified maximum limit
    pub fn increment_with_limits(self, max: f32) -> Self {
        match self {
            Zoom::Full => Zoom::Value(0.1), // From full view, start at minimum zoom
            Zoom::Value(value) => {
                let new_value = if value < 1.0 {
                    // When zoomed out, increment in 0.1 steps: 0.1 -> 0.2 -> ... -> 1.0
                    (value + 0.1).min(1.0)
                } else {
                    // When zoomed in, increment in 1.0 steps: 1.0 -> 2.0 -> 3.0 ... -> max
                    (value + 1.0).min(max)
                };
                Zoom::Value(new_value)
            }
        }
    }

    /// Decrement zoom with a specified minimum limit
    pub fn decrement_with_limits(self, min: f32) -> Self {
        match self {
            Zoom::Full => Zoom::Full, // Already at maximum zoom out
            Zoom::Value(value) => {
                if value <= min {
                    // If at or below minimum, go to full view
                    Zoom::Full
                } else if value <= 1.0 {
                    // When at 1x or below, decrement in 0.1 steps down to min
                    let new_value = (value - 0.1).max(min);
                    if new_value <= min {
                        Zoom::Full
                    } else {
                        Zoom::Value(new_value)
                    }
                } else {
                    // When zoomed in, decrement in 1.0 steps: max -> ... -> 2.0 -> 1.0
                    Zoom::Value((value - 1.0).max(1.0))
                }
            }
        }
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self::Value(1.0)
    }
}
