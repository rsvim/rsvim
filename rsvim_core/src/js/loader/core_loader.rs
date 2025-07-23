//! Core module loader.

use crate::js::loader::ModuleLoader;
use crate::js::module::{CORE_MODULES, ModulePath, ModuleSource};
use crate::prelude::*;

#[derive(Default)]
/// For core module loader, its module path is the same with the specifier.
///
/// To indicate core modules, they use a special pattern 'rsvim:ext' as its prefix. For example:
///
/// ```javascript
/// const net = import "rsvim:ext/net";
/// const fs = import "rsvim:ext/fs";
/// const process = import "rsvim:ext/process";
/// ```
///
/// But these are just some examples (to show how core module specifiers are different from other
/// modules). For now we don't support any core modules.
pub struct CoreModuleLoader;

impl ModuleLoader for CoreModuleLoader {
  /// Resolve module path by its specifier.
  fn resolve(
    &self,
    _base: Option<&str>,
    specifier: &str,
  ) -> AnyResult<ModulePath> {
    assert!(CORE_MODULES.contains_key(specifier));
    Ok(specifier.to_string())
  }

  /// Load module source by its module path.
  fn load(&self, module_path: &str) -> AnyResult<ModuleSource> {
    assert!(CORE_MODULES.contains_key(module_path));
    Ok(CORE_MODULES.get(module_path).unwrap().to_string())
  }
}
