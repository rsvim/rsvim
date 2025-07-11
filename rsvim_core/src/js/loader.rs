//! Js module loader.

use crate::js::module::ModulePath;
use crate::js::module::ModuleSource;
// use crate::js::transpiler::Jsx;
// use crate::js::transpiler::Wasm;
use crate::prelude::*;

// Re-export
pub use core_loader::CoreModuleLoader;
pub use fs_loader::FsModuleLoader;

// use sha::sha1::Sha1;
// use sha::utils::Digest;
// use sha::utils::DigestExt;
// use url::Url;

pub mod core_loader;
pub mod fs_loader;
// pub mod url_loader;

#[cfg(test)]
mod fs_loader_tests;

/// Defines the interface of a module loader.
pub trait ModuleLoader {
  /// Resolve the module's path by its specifier.
  fn resolve(&self, base: Option<&str>, specifier: &str) -> AnyResult<ModulePath>;

  /// Load the module path by its specifier.
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource>;
}

/// Resolves an import using the appropriate loader.
/// Returns full path on local file system.
pub fn resolve_import(
  base: Option<&str>,
  specifier: &str,
  ignore_core_modules: bool,
  import_map: Option<ImportMap>,
) -> AnyResult<ModulePath> {
  // Use import-maps if available.
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

/// Loads an import using the appropriate loader.
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

pub async fn load_import_async(specifier: &str, skip_cache: bool) -> AnyResult<ModuleSource> {
  load_import(specifier, skip_cache)
}
