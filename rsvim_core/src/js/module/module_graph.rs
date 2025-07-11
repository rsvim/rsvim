//! Module graph.
//!
//! Module graph maintains the relationships between a module and its dependencies tree.
//!
//! The dependency is created by the `require` or `import` keyword. And there're two kinds of
//! import: static and dynamic. For example:
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

use crate::js::module::{EsModule, ModuleStatus};

use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;
// use url::Url;

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
///
/// A module's tree dependency graph.
pub struct ModuleGraph {
  pub kind: ImportKind,
  pub root_rc: Rc<RefCell<EsModule>>,
  pub same_origin: LinkedList<v8::Global<v8::PromiseResolver>>,
}

impl ModuleGraph {
  // Initializes a new graph resolving a static import.
  pub fn static_import(path: &str) -> ModuleGraph {
    // Create an ES module instance.
    let module = Rc::new(RefCell::new(EsModule {
      path: path.into(),
      status: ModuleStatus::Fetching,
      dependencies: vec![],
      exception: Rc::new(RefCell::new(None)),
      is_dynamic_import: false,
    }));

    Self {
      kind: ImportKind::Static,
      root_rc: module,
      same_origin: LinkedList::new(),
    }
  }

  // Initializes a new graph resolving a dynamic import.
  pub fn dynamic_import(path: &str, promise: v8::Global<v8::PromiseResolver>) -> ModuleGraph {
    // Create an ES module instance.
    let module = Rc::new(RefCell::new(EsModule {
      path: path.into(),
      status: ModuleStatus::Fetching,
      dependencies: vec![],
      exception: Rc::new(RefCell::new(None)),
      is_dynamic_import: true,
    }));

    Self {
      kind: ImportKind::Dynamic(promise),
      root_rc: module,
      same_origin: LinkedList::new(),
    }
  }
}
