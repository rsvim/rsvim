use crate::prelude::*;
use crate::ui::tree::internal::inode::*;
use itertools::Itertools;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use taffy::Style;
use taffy::TaffyResult;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;
use taffy::prelude::TaffyAuto;

#[derive(Debug, Clone)]
pub struct Itree<T>
where
  T: Inodeable,
{
  // Nodes collection, maps from node ID to its node struct.
  nodes: FoldMap<TreeNodeId, T>,

  // Maps parent and children edges. The parent edge weight is negative,
  // children edges are positive. The edge weight of each child is increased
  // with the order when they are inserted, i.e. the first child has the lowest
  // edge weight, the last child has the highest edge weight.
  relationships: RefCell<Relationships>,
}
