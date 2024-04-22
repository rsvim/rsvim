use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

#[derive(Debug)]
pub struct Node {
  pub parent: Option<Rc<RefCell<Node>>>,
  pub children: LinkedList<Rc<RefCell<Node>>>,
}
