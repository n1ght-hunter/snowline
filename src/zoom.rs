//! Shared zoom functionality for all graph types

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zoom(f32);

impl Zoom {
    /// Create a new zoom level with the given value
    pub fn new(value: f32) -> Self {
        Self(value.max(0.1)) // Ensure minimum zoom
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
        let new_value = if self.0 < 1.0 {
            // When zoomed out, increment in 0.1 steps: 0.1 -> 0.2 -> ... -> 1.0
            (self.0 + 0.1).min(1.0)
        } else {
            // When zoomed in, increment in 1.0 steps: 1.0 -> 2.0 -> 3.0 ... -> max
            (self.0 + 1.0).min(max)
        };
        Self(new_value)
    }

    /// Decrement zoom with a specified minimum limit
    pub fn decrement_with_limits(self, min: f32) -> Self {
        let new_value = if self.0 <= 1.0 {
            // When at 1x or below, decrement in 0.1 steps down to min
            (self.0 - 0.1).max(min)
        } else {
            // When zoomed in, decrement in 1.0 steps: max -> ... -> 2.0 -> 1.0
            (self.0 - 1.0).max(1.0)
        };
        Self(new_value)
    }

    /// Get the current zoom value
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self(1.0)
    }
}
