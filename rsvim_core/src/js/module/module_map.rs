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

use crate::js::module::ModulePath;
use crate::js::module::ModuleStatus;
use crate::js::module::es_module::*;
use crate::prelude::*;
use std::fmt::Debug;

#[derive(Clone)]
/// Import kind.
pub enum ImportKind {
  // Loading static imports.
  Static,
  // Loading a dynamic import.
  Dynamic(v8::Global<v8::PromiseResolver>),
}

/// Module graph.
pub struct ModuleGraph {
  kind: ImportKind,
  root_rc: EsModuleRc,
  same_origin: Vec<v8::Global<v8::PromiseResolver>>,
}

rc_refcell_ptr!(ModuleGraph);

impl Debug for ModuleGraph {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ModuleGraph")
      .field(
        "kind",
        match self.kind {
          ImportKind::Static => &"Static",
          ImportKind::Dynamic(_) => &"Dynamic",
        },
      )
      .field("root_rc", &self.root_rc)
      .field(
        "same_origin",
        &format!("Vec<v8::PromiseResolver>({})", self.same_origin.len()),
      )
      .finish()
  }
}

impl ModuleGraph {
  pub fn kind(&self) -> &ImportKind {
    &self.kind
  }

  pub fn root_rc(&self) -> EsModuleRc {
    self.root_rc.clone()
  }

  pub fn same_origin(&self) -> &Vec<v8::Global<v8::PromiseResolver>> {
    &self.same_origin
  }

  pub fn same_origin_mut(
    &mut self,
  ) -> &mut Vec<v8::Global<v8::PromiseResolver>> {
    &mut self.same_origin
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
      same_origin: vec![],
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
      same_origin: vec![],
    }
  }
}

/// Module map.
/// It maintains all the modules inside js runtime, including already resolved and pending
/// fetching.
pub struct ModuleMap {
  // Entry point of runtime execution, this is the `rsvim.{js,ts}`
  // configuration entry point for Rsvim.
  main: Option<ModulePath>,

  // Maps from "Module Path" to "v8 Module".
  index: HashMap<ModulePath, v8::Global<v8::Module>>,

  // Module status.
  pub seen: HashMap<ModulePath, ModuleStatus>,

  // Pending modules.
  pub pending: Vec<ModuleGraphRc>,
}

impl Debug for ModuleMap {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ModuleMap")
      .field("main", &self.main)
      .field(
        "index",
        &self
          .index
          .keys()
          .map(|k| (k.clone(), "v8::Module".to_string()))
          .collect::<HashMap<String, String>>(),
      )
      .field("seen", &self.seen)
      .field("pending", &self.pending)
      .finish()
  }
}

impl ModuleMap {
  /// Creates a global module map.
  pub fn new() -> ModuleMap {
    Self {
      main: None,
      index: HashMap::new(),
      seen: HashMap::new(),
      pending: vec![],
    }
  }

  pub fn main(&self) -> &Option<ModulePath> {
    &self.main
  }

  /// Add a compiled v8 module to the cache.
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

  /// Returns a compiled v8 module.
  pub fn get(&self, key: &str) -> Option<v8::Global<v8::Module>> {
    self.index.get(key).cloned()
  }

  /// Whether a v8 module already resolved.
  pub fn contains(&self, key: &str) -> bool {
    self.index.contains_key(key)
  }

  #[cfg(test)]
  /// Whether a v8 module already resolved, compare by key suffix.
  pub fn get_by_suffix(
    &self,
    key: &str,
  ) -> Option<(&ModulePath, &v8::Global<v8::Module>)> {
    use normpath::PathExt;

    self.index.iter().find(|(k, _v)| {
      Path::new(k)
        .normalize()
        .unwrap()
        .ends_with(Path::new(key).normalize().unwrap())
        || Path::new(key)
          .normalize()
          .unwrap()
          .ends_with(Path::new(*k).normalize().unwrap())
    })
  }

  /// Returns a specifier by a v8 module ID.
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
