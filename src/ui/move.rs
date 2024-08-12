//! Widget movement.

#[derive(Debug, Default, Copy, Clone)]
/// Bounded move, the movement will not take effect if it's already on the edge.
pub struct BoundedMove {}

#[derive(Debug, Default, Copy, Clone)]
/// Unbounded move, the movement is logically always effective.
pub struct UnboundedMove {}
