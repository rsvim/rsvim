//! Module map and module graph.
//!
//! # Terms
//!
//! - Module graph: it maintains the relationships between a module and its dependencies tree.
//! - Module map: it maintains all the dependencies loaded into the js rutime, i.e. the rsvim
//!   editor.
//!
//! # Module
//!
//! In the most popular js runtime [node.js](https://nodejs.org), a module is loaded by the keyword
//! `require` (Common JS) or `import` (ECMAScript Module). And there're two kinds of imports:
//! static and dynamic. For example:
//!
//! Case-1: Static Import
//!
//! ```javascript
//! // ES Module
//! const _ = import "lodash";
//!
//! // Common JS
//! const _ = require("lodash");
//! ```
//!
//! // Case-2: Dynamic Import
//!
//! ```javascript
//! // Wait for import complete.
//! const _ = await import("lodash");
//!
//! // Trigger callbacks on import complete.
//! import("lodash")
//!   .then((_) => {})
//!   .catch((err) => {});
//! ```
//!
//! Static import runs synchronizely, dynamic import runs asynchronizely.

use crate::js::module::es_module::*;
use crate::js::module::{ModulePath, ModuleStatus};
use crate::prelude::*;

use std::cell::{Ref, RefCell, RefMut};
use std::collections::LinkedList;

#[derive(Debug, Clone)]
/// Import kind.
pub enum ImportKind {
  // Loading static imports.
  Static,
  // Loading a dynamic import.
  Dynamic(v8::Global<v8::PromiseResolver>),
}

#[derive(Debug)]
/// Module graph.
pub struct ModuleGraph {
  kind: ImportKind,
  root_rc: EsModuleRc,
  same_origin: LinkedList<v8::Global<v8::PromiseResolver>>,
}

rc_refcell_ptr!(ModuleGraph);

impl ModuleGraph {
  pub fn kind(&self) -> &ImportKind {
    &self.kind
  }

  pub fn root_rc(&self) -> EsModuleRc {
    self.root_rc.clone()
  }

  pub fn same_origin(&self) -> &LinkedList<v8::Global<v8::PromiseResolver>> {
    &self.same_origin
  }
}

impl ModuleGraph {
  // Initializes a new graph resolving a static import.
  pub fn static_import(path: &str) -> ModuleGraph {
    // Create an ES module instance.
    let module = EsModule::to_rc(EsModule::new(
      path.into(),
      ModuleStatus::Fetching,
      vec![],
      None,
      false,
    ));

    Self {
      kind: ImportKind::Static,
      root_rc: module,
      same_origin: LinkedList::new(),
    }
  }

  // Initializes a new graph resolving a dynamic import.
  pub fn dynamic_import(
    path: &str,
    promise: v8::Global<v8::PromiseResolver>,
  ) -> ModuleGraph {
    // Create an ES module instance.
    let module = EsModule::to_rc(EsModule::new(
      path.into(),
      ModuleStatus::Fetching,
      vec![],
      None,
      true,
    ));

    Self {
      kind: ImportKind::Dynamic(promise),
      root_rc: module,
      same_origin: LinkedList::new(),
    }
  }
}

/// Module map.
/// It maintains all the modules inside js runtime, including already resolved and pending
/// fetching.
pub struct ModuleMap {
  main: Option<ModulePath>,
  index: HashMap<ModulePath, v8::Global<v8::Module>>,
  seen: RefCell<HashMap<ModulePath, ModuleStatus>>,
  pending: RefCell<Vec<ModuleGraphRc>>,
}

impl ModuleMap {
  pub fn main(&self) -> &Option<ModulePath> {
    &self.main
  }

  pub fn index(&self) -> &HashMap<ModulePath, v8::Global<v8::Module>> {
    &self.index
  }

  pub fn seen(&self) -> Ref<'_, HashMap<ModulePath, ModuleStatus>> {
    self.seen.borrow()
  }

  pub fn seen_mut(&self) -> RefMut<'_, HashMap<ModulePath, ModuleStatus>> {
    self.seen.borrow_mut()
  }

  pub fn pending(&self) -> Ref<'_, Vec<ModuleGraphRc>> {
    self.pending.borrow()
  }

  pub fn pending_mut(&self) -> RefMut<'_, Vec<ModuleGraphRc>> {
    self.pending.borrow_mut()
  }
}

impl ModuleMap {
  // Creates a new module-map instance.
  pub fn new() -> ModuleMap {
    Self {
      main: None,
      index: HashMap::new(),
      seen: RefCell::new(HashMap::new()),
      pending: RefCell::new(vec![]),
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

  // // Returns if there are still pending imports to be loaded.
  // pub fn has_pending_imports(&self) -> bool {
  //   !self.pending.is_empty()
  // }

  // Returns a v8 module reference from me module-map.
  pub fn get(&self, key: &str) -> Option<v8::Global<v8::Module>> {
    self.index.get(key).cloned()
  }

  // Returns a specifier by a v8 module.
  // FIXME: This method has performance issue, make it `O(1)` instead of
  // `O(N)`.
  pub fn get_path(&self, module: v8::Global<v8::Module>) -> Option<ModulePath> {
    self
      .index
      .iter()
      .find(|(_, m)| **m == module)
      .map(|(p, _)| p.clone())
  }
}

impl Default for ModuleMap {
  fn default() -> Self {
    ModuleMap::new()
  }
}
