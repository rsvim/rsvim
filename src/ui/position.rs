#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position<T> {
  pub x: T,
  pub y: T,
}

impl<T> Position<T> {
  pub fn new(x: T, y: T) -> Self {
    Position { x, y }
  }

  pub fn swap(self) -> Self {
    Position::new(self.y, self.x)
  }
}

// Relative position.
pub type RelPosition = Position<i32>;

// Absolute position.
pub type AbsPosition = Position<u32>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_all_zero_on_relpos_default() {
    let p1: RelPosition = Default::default();
    let p2 = RelPosition::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_relpos_swap() {
    let p1 = RelPosition::new(1, 2);
    assert_eq!(p1.swap(), RelPosition::new(2, 1));
  }

  #[test]
  fn should_all_zero_on_abspos_default() {
    let p1: AbsPosition = Default::default();
    let p2 = AbsPosition::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_abspos_swap() {
    let p1 = AbsPosition::new(1, 2);
    assert_eq!(p1.swap(), AbsPosition::new(2, 1));
  }
}
