/// Shared trait to map a datapoint into an `f64` for rendering.
/// Implementations may freely use references to avoid cloning.
pub trait ValueMapper<T> {
    fn map(&self, value: &T) -> f64;
}

/// Default mapper: uses `Into<f64>` on Copy values; zero allocations and no cloning of T
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultMap;

impl<T> ValueMapper<T> for DefaultMap
where
    T: Copy + Into<f64>,
{
    fn map(&self, value: &T) -> f64 {
        (*value).into()
    }
}

/// Blanket impl: allow closures `Fn(&T) -> f64` as mappers without cloning.
impl<T, F> ValueMapper<T> for F
where
    F: Fn(&T) -> f64,
{
    fn map(&self, value: &T) -> f64 {
        self(value)
    }
}
