//! ECMAScript (ES) module, i.e. the module specified by keyword `import`.

use crate::js::module::{ModulePath, ModuleStatus};
use crate::prelude::*;

#[derive(Debug)]
/// ES Module.
pub struct EsModule {
  /// Module path on local file system.
  path: ModulePath,
  /// Module import status.
  status: ModuleStatus,
  /// Maps the module itself to all its dependencies.
  dependencies: Vec<EsModuleRc>,
  /// Exceptions when import.
  exception: Option<String>,
  /// Whether this module is dynamically import.
  is_dynamic_import: bool,
}

rc_refcell_ptr!(EsModule);

impl EsModule {
  pub fn new(
    path: ModulePath,
    status: ModuleStatus,
    dependencies: Vec<EsModuleRc>,
    exception: Option<String>,
    is_dynamic_import: bool,
  ) -> Self {
    Self {
      path,
      status,
      dependencies,
      exception,
      is_dynamic_import,
    }
  }

  pub fn path(&self) -> &ModulePath {
    &self.path
  }

  pub fn status(&self) -> ModuleStatus {
    self.status
  }

  pub fn set_status(&mut self, status: ModuleStatus) {
    self.status = status;
  }

  pub fn dependencies(&self) -> &Vec<EsModuleRc> {
    &self.dependencies
  }

  pub fn dependencies_mut(&mut self) -> &mut Vec<EsModuleRc> {
    &mut self.dependencies
  }

  pub fn exception(&self) -> &Option<String> {
    &self.exception
  }

  pub fn exception_mut(&mut self) -> &mut Option<String> {
    &mut self.exception
  }

  pub fn is_dynamic_import(&self) -> bool {
    self.is_dynamic_import
  }
}

impl EsModule {
  // Traverses the dependency tree to check if the module is ready.
  pub fn fast_forward(
    &mut self,
    seen_modules: &mut HashMap<ModulePath, ModuleStatus>,
  ) {
    // If the module is ready, no need to check the sub-tree.
    if self.status == ModuleStatus::Ready {
      return;
    }

    // If it's a duplicate module we need to check the module status cache.
    if self.status == ModuleStatus::Duplicate {
      let status_ref = seen_modules.get(&self.path).unwrap();
      if status_ref == &ModuleStatus::Ready {
        self.status = ModuleStatus::Ready;
      }
      return;
    }

    // Fast-forward all dependencies.
    self
      .dependencies
      .iter_mut()
      .for_each(|dep| dep.borrow_mut().fast_forward(seen_modules));

    // The module is compiled and has 0 dependencies.
    if self.dependencies.is_empty() && self.status == ModuleStatus::Resolving {
      self.status = ModuleStatus::Ready;
      seen_modules.insert(self.path.clone(), self.status);
      return;
    }

    // At this point, the module is still being fetched...
    if self.dependencies.is_empty() {
      return;
    }

    if !self
      .dependencies
      .iter_mut()
      .map(|m| m.borrow().status)
      .any(|status| status != ModuleStatus::Ready)
    {
      self.status = ModuleStatus::Ready;
      seen_modules.insert(self.path.clone(), self.status);
    }
  }
}
