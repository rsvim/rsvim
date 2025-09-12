//! Js module loader.

use crate::js::module::{ModulePath, ModuleSource};
use crate::prelude::*;

// Re-export
pub use core_loader::CoreModuleLoader;
pub use fs_loader::{AsyncFsModuleLoader, FsModuleLoader};

use async_trait::async_trait;

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
pub trait ModuleLoader {
  /// Resolve module path by its specifier.
  ///
  /// - For core module loader, the module path is always same with its specifier (core modules
  ///   don't have a file path on local filesystem).
  /// - For fs module loader, it returns the full file path on local filesystem.
  /// - For url module loader, the specifier is a url (mostly http/https) which indicates a remote
  ///   location that can download the resource.
  ///
  /// NOTE: This API (and all 3 loaders) are synchronized, include the url module loader's
  /// downloading process.
  fn resolve(
    &self,
    base: Option<&str>,
    specifier: &str,
  ) -> AnyResult<ModulePath>;

  /// Load the module source by its module path.
  ///
  /// For url module loader, it will first download the resource to local filesystem as local file
  /// cache, then read the cache contents and return as module source code.
  fn load(&self, module_path: &str) -> AnyResult<ModuleSource>;
}

#[async_trait]
/// Async [`ModuleLoader`].
///
/// NOTE: This is only allow to use in event loop, i.e. with tokio runtime, not
/// in js runtime.
pub trait AsyncModuleLoader {
  async fn load(&self, module_path: &str) -> AnyResult<ModuleSource>;
}
