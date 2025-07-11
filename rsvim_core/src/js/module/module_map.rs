//! Module map.

use crate::js::module::{ModuleGraph, ModulePath, ModuleStatus};
use crate::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

/// Module map, it maintains all the modules inside js runtime, including already resolved, loaded
/// and pending fetching. A module will be only loaded for only once and been cached, thus avoid
/// duplicated loading if it is used in multiple places.
pub struct ModuleMap {
  pub main: Option<ModulePath>,
  pub index: HashMap<ModulePath, v8::Global<v8::Module>>,
  pub seen: HashMap<ModulePath, ModuleStatus>,
  pub pending: Vec<Rc<RefCell<ModuleGraph>>>,
}

impl ModuleMap {
  // Creates a new module-map instance.
  pub fn new() -> ModuleMap {
    Self {
      main: None,
      index: HashMap::new(),
      seen: HashMap::new(),
      pending: vec![],
    }
  }

  // Inserts a compiled ES module to the map.
  pub fn insert(&mut self, path: &str, module: v8::Global<v8::Module>) {
    // No main module has been set, so let's update the value.
    if self.main.is_none() && std::fs::metadata(path).is_ok() {
      self.main = Some(path.into());
    }
    self.index.insert(path.into(), module);
  }

  // Returns if there are still pending imports to be loaded.
  pub fn has_pending_imports(&self) -> bool {
    !self.pending.is_empty()
  }

  // Returns a v8 module reference from me module-map.
  pub fn get(&self, key: &str) -> Option<v8::Global<v8::Module>> {
    self.index.get(key).cloned()
  }

  // Returns a specifier by a v8 module.
  pub fn get_path(&self, module: v8::Global<v8::Module>) -> Option<ModulePath> {
    self
      .index
      .iter()
      .find(|(_, m)| **m == module)
      .map(|(p, _)| p.clone())
  }

  // Returns the main entry point.
  pub fn main(&self) -> Option<ModulePath> {
    self.main.clone()
  }
}

impl Default for ModuleMap {
  fn default() -> Self {
    ModuleMap::new()
  }
}
