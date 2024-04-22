#[derive(Debug)]
pub struct State {
  pub cols: u16,
  pub rows: u16,
}

impl State {
  pub fn new(cols: u16, rows: u16) -> State {
    State { cols, rows }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new() {
    let stat = State::new(1, 2);
    assert_eq!(stat.cols, 1);
    assert_eq!(stat.rows, 2);
  }
}
