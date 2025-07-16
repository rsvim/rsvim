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
/// modules). For now we don't have any core modules, or say we don't use core modules.
pub struct CoreModuleLoader;

impl ModuleLoader for CoreModuleLoader {
  fn resolve(&self, _: Option<&str>, specifier: &str) -> AnyResult<ModulePath> {
    assert!(CORE_MODULES.contains_key(specifier));
    Ok(specifier.to_string())
  }

  fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    // Since any errors will be caught at the resolve stage, we can
    // go ahead an unwrap the value with no worries.
    Ok(CORE_MODULES.get(specifier).unwrap().to_string())
  }
}
