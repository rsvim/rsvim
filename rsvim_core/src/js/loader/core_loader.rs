//! Core module loader.

use crate::js::loader::ModuleLoader;
use crate::js::module::CORE_MODULES;
use crate::js::module::ModulePath;
use crate::js::module::ModuleSource;
use crate::prelude::*;

#[derive(Default)]
pub struct CoreModuleLoader;

impl ModuleLoader for CoreModuleLoader {
  fn resolve(&self, _: Option<&str>, specifier: &str) -> AnyResult<ModulePath> {
    assert!(CORE_MODULES().contains_key(specifier));
    Ok(specifier.to_string())
  }

  fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    // Since any errors will be caught at the resolve stage, we can
    // go ahead an unwrap the value with no worries.
    Ok(CORE_MODULES().get(specifier).unwrap().to_string())
  }
}
