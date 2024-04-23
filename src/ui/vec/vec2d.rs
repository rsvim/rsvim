#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec2d {
  pub x: i32,
  pub y: i32,
}

impl Vec2d {
  pub fn new(x: i32, y: i32) -> Self {
    Vec2d { x, y }
  }

  pub fn swap(self) -> Self {
    Vec2d::new(self.y, self.x)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_all_zero_on_vec2d_default() {
    let p1: Vec2d = Default::default();
    let p2 = Vec2d::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_vec2d_swap() {
    let p1 = Vec2d::new(1, 2);
    assert_eq!(p1.swap(), Vec2d::new(2, 1));
  }
}
