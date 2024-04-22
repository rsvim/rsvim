#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos<T> {
  pub x: T,
  pub y: T,
}

impl<T> Pos<T> {
  pub fn new(x: T, y: T) -> Self {
    Pos { x, y }
  }

  pub fn swap(self) -> Self {
    Pos::new(self.y, self.x)
  }
}

// Relative position.
pub type RelPos = Pos<isize>;

// Absolute position.
pub type AbsPos = Pos<usize>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_all_zero_on_relpos_default() {
    let p1: RelPos = Default::default();
    let p2 = RelPos::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_relpos_swap() {
    let p1 = RelPos::new(1, 2);
    assert_eq!(p1.swap(), RelPos::new(2, 1));
  }

  #[test]
  fn should_all_zero_on_abspos_default() {
    let p1: AbsPos = Default::default();
    let p2 = AbsPos::new(0, 0);
    assert_eq!(p1, p2);
  }

  #[test]
  fn should_reverse_after_abspos_swap() {
    let p1 = AbsPos::new(1, 2);
    assert_eq!(p1.swap(), AbsPos::new(2, 1));
  }
}
