//! Core module loader.

use crate::js::loader::ModuleLoader;
use crate::js::module::{CORE_MODULES, ModulePath, ModuleSource};
use crate::prelude::*;

#[derive(Default)]
/// For core module loader, its module path is the same with module specifier. And all core modules
/// have a special prefix 'rsvim:' to indicate them. For example:
///
/// ```javascript
/// const net = import "rsvim:ext/net";
/// const fs = import "rsvim:ext/fs";
/// const process = import "rsvim:ext/process";
/// ```
///
/// But these are just some examples (to show how core module specifiers are different from other
/// modules). For now we don't have any core modules, or say we don't use core modules.
pub struct CoreModuleLoader;

impl ModuleLoader for CoreModuleLoader {
  /// Resolve module path, which is same with specifier.
  fn resolve(&self, _: Option<&str>, specifier: &str) -> AnyResult<ModulePath> {
    assert!(CORE_MODULES().contains_key(specifier));
    Ok(specifier.to_string())
  }

  // Load module source by its path.
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    assert!(CORE_MODULES().contains_key(specifier));
    Ok(CORE_MODULES().get(specifier).unwrap().to_string())
  }
}
