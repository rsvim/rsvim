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

/// Module loader.
///
/// There are 3 kinds of module loaders:
/// 1. Core module loader, provide builtin modules.
/// 2. Fs module loader, provide modules on local filesystem.
/// 3. URL module loader, provide remote modules on network.
///
/// TODO:
/// For now we only implement the fs module loader, in the future we may want to implement other
/// loaders.
pub trait ModuleLoader {
  /// Resolve module path by its specifier.
  ///
  /// - For core module loader, the module path is always same with its specifier (core modules
  ///   don't have a file path on local filesystem).
  /// - For fs module loader, it returns the full file path on local filesystem.
  /// - For url module loader, the specifier is a url (mostly http protocol) which indicates a
  ///   remote location that can download the resource. Url module loader will first download the
  ///   resource to local filesystem as a local file cache, then returns the cached full file path.
  ///
  /// NOTE: This API (and all 3 loaders) are synchronized, include the url module loader's
  /// downloading process.
  fn resolve(&self, base: Option<&str>, specifier: &str) -> AnyResult<ModulePath>;

  /// Load the module source by its module path.
  fn load(&self, module_path: &str) -> AnyResult<ModuleSource>;
}
