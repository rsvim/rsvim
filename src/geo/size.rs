//! Rectangle size: height/width, also known as rows/columns.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// Rectangle size.
pub struct Size {
  /// Also known as rows.
  pub height: usize,
  /// Also known as columns.
  pub width: usize,
}

impl Size {
  /// Make new [size](Size) from [height](Size::height) and [width](Size::width).
  pub fn new(height: usize, width: usize) -> Self {
    Size { height, width }
  }

  /// The area of the [size](Size), i.e. [height](Size::height) * [width](Size::width).
  pub fn area(&self) -> usize {
    self.height * self.width
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_size_area() {
    let sz = Size::new(5, 10);
    assert_eq!(sz.height, 5);
    assert_eq!(sz.width, 10);
    assert_eq!(sz.area(), 5 * 10);
  }
}
