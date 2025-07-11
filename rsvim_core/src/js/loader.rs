//! Js module loader.

use crate::js::module::{ModulePath, ModuleSource};
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
