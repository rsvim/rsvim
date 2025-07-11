//! Js module loader.

use crate::js::module::{CORE_MODULES, ImportMap, ModulePath, ModuleSource};
use crate::prelude::*;

// Re-export
pub use core_loader::CoreModuleLoader;
pub use fs_loader::FsModuleLoader;

pub mod core_loader;
pub mod fs_loader;

#[cfg(test)]
mod fs_loader_tests;

/// Defines the interface of a module loader.
pub trait ModuleLoader {
  /// Resolve the module's path by its specifier.
  fn resolve(&self, base: Option<&str>, specifier: &str) -> AnyResult<ModulePath>;

  /// Load the module path by its specifier.
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource>;
}

/// Resolves module path by its specifier.
///
/// The `base` parameter is current module's local filesystem path, all its dependent modules'
/// filesystem path should be relatively based on the same directory that contains the root module,
/// i.e. current module.
///
/// The `import_map` is an optional user provided map that overwrite default module loader, see
/// [`ImportMap`].
///
/// # Returns
///
/// It returns full path on local filesystem.
pub fn resolve_import(
  base: Option<&str>,
  specifier: &str,
  ignore_core_modules: bool,
  import_map: Option<ImportMap>,
) -> AnyResult<ModulePath> {
  // Use import-maps if available.
  // FIXME: This is not supported now.
  let specifier = match import_map {
    Some(map) => map.lookup(specifier).unwrap_or_else(|| specifier.into()),
    None => specifier.into(),
  };

  // Look the params and choose a loader, then resolve module.
  let is_core_module_import = CORE_MODULES().contains_key(specifier.as_str());
  if is_core_module_import && !ignore_core_modules {
    CoreModuleLoader {}.resolve(base, &specifier)
  } else {
    FsModuleLoader {}.resolve(base, &specifier)
  }
}

/// Loads module source by its specifier.
pub fn load_import(specifier: &str, _skip_cache: bool) -> AnyResult<ModuleSource> {
  // // Look the params and choose a loader.
  // let loader: Box<dyn ModuleLoader> = match (
  //   CORE_MODULES().contains_key(specifier),
  //   WINDOWS_REGEX().is_match(specifier),
  //   Url::parse(specifier).is_ok(),
  // ) {
  //   (true, _, _) => Box::new(CoreModuleLoader),
  //   (_, true, _) => Box::new(FsModuleLoader),
  //   (_, _, true) => Box::new(UrlModuleLoader { skip_cache }),
  //   _ => Box::new(FsModuleLoader),
  // };
  //
  // // Load module.
  // loader.load(specifier)

  let is_core_module_import = CORE_MODULES().contains_key(specifier);
  if is_core_module_import {
    CoreModuleLoader {}.load(specifier)
  } else {
    FsModuleLoader {}.load(specifier)
  }

  // // We don't actually have core modules
  // FsModuleLoader {}.load(specifier)
}

/// FIXME: Not support for now.
pub async fn load_import_async(specifier: &str, skip_cache: bool) -> AnyResult<ModuleSource> {
  load_import(specifier, skip_cache)
}
