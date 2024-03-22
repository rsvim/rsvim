pub fn example() {
  println!("Hello Rsvim!");
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_example() {
    example();
    assert_eq!(1, 1);
  }
}
