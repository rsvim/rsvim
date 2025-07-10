//! Js module loader.

use crate::js::module::ModulePath;
use crate::js::module::ModuleSource;
// use crate::js::transpiler::Jsx;
// use crate::js::transpiler::Wasm;
use crate::prelude::*;

// use sha::sha1::Sha1;
// use sha::utils::Digest;
// use sha::utils::DigestExt;
// use url::Url;

pub mod core_loader;
pub mod fs_loader;
pub mod url_loader;

#[cfg(test)]
mod fs_loader_tests;

static FILE_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "json", "json5", "wasm"];

/// Defines the interface of a module loader.
pub trait ModuleLoader {
  /// Resolve the module's path by its specifier.
  fn resolve(&self, base: Option<&str>, specifier: &str) -> AnyResult<ModulePath>;

  /// Load the module path by its specifier.
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource>;
}
