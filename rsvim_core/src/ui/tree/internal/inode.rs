//! Internal tree node.

use crate::ui::tree::TreeNodeId;

pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> TreeNodeId;

  fn depth(&self) -> usize;

  fn set_depth(&mut self, depth: usize);

  fn zindex(&self) -> usize;

  fn set_zindex(&mut self, zindex: usize);

  fn shape(&self) -> &IRect;

  fn set_shape(&mut self, shape: &IRect);

  fn actual_shape(&self) -> &U16Rect;

  fn set_actual_shape(&mut self, actual_shape: &U16Rect);

  fn enabled(&self) -> bool;

  fn set_enabled(&mut self, enabled: bool);

  fn visible(&self) -> bool;

  fn set_visible(&mut self, visible: bool);
}
