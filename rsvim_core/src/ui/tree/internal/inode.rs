//! Internal tree node.

use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::itree::IrelationshipRc;
use taffy::Layout;
use taffy::Style;
use taffy::TaffyResult;

pub trait Inodeable: Sized + Clone + std::fmt::Debug {
  fn id(&self) -> TreeNodeId;

  fn relationship(&self) -> IrelationshipRc;

  fn layout(&self) -> TaffyResult<Layout>;

  fn style(&self) -> TaffyResult<Style>;

  fn shape(&self) -> &IRect;

  fn set_shape(&mut self, shape: &IRect);

  fn actual_shape(&self) -> &U16Rect;

  fn set_actual_shape(&mut self, actual_shape: &U16Rect);

  fn enabled(&self) -> bool;

  fn set_enabled(&mut self, enabled: bool);

  fn visible(&self) -> bool;

  fn set_visible(&mut self, visible: bool);
}
