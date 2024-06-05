use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::uuid;

pub struct Window {
  id: usize,
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  visible: bool,
  enabled: bool,
}

impl Window {
  pub fn new(size: Size) -> Self {
    Window {
      id: uuid::next(),
      offset: IPos::new(0, 0),
      abs_offset: UPos::new(0, 0),
      size,
      visible: true,
      enabled: true,
    }
  }
}
