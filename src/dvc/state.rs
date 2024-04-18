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
