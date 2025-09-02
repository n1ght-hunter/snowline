//! State management for bar graphs

#[derive(Debug, Clone, Default)]
pub struct BarGraphState {
    pub hovered_bar: Option<usize>,
}
