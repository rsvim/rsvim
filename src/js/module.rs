//! Js modules.

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::rc::Rc;

// pub mod transpiler;

#[derive(Debug, Clone)]
pub enum ImportKind {
  // Loading static imports.
  Static,
  // Loading a dynamic import.
  Dynamic(v8::Global<v8::PromiseResolver>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
  // Indicates the module is being fetched.
  Fetching,
  // Indicates the dependencies are being fetched.
  Resolving,
  // Indicates the module has ben seen before.
  Duplicate,
  // Indicates the modules is resolved.
  Ready,
}

#[derive(Debug)]
pub struct EsModule {
  pub path: ModulePath,
  pub status: ModuleStatus,
  pub dependencies: Vec<Rc<RefCell<EsModule>>>,
  pub exception: Rc<RefCell<Option<String>>>,
  pub is_dynamic_import: bool,
}

#[derive(Debug)]
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

pub type ModulePath = String;
pub type ModuleSource = String;

pub struct ModuleMap {
  pub main: Option<ModulePath>,
  pub index: HashMap<ModulePath, v8::Global<v8::Module>>,
  pub seen: HashMap<ModulePath, ModuleStatus>,
  pub pending: Vec<Rc<RefCell<ModuleGraph>>>,
}
