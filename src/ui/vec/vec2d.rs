#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec2d<T> {
  pub x: T,
  pub y: T,
}

impl<T> Vec2d<T> {
  pub fn new(x: T, y: T) -> Self {
    Vec2d { x, y }
  }

  pub fn swap(self) -> Self {
    Vec2d::new(self.y, self.x)
  }
}

pub type RelVec2d = Vec2d<i32>;
pub type AbsVec2d = Vec2d<u32>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_all_zero_on_relvec2d_default() {
    let p1: RelVec2d = Default::default();
    let p2 = RelVec2d::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_relvec2d_swap() {
    let p1 = RelVec2d::new(1, 2);
    assert_eq!(p1.swap(), RelVec2d::new(2, 1));
  }
}
