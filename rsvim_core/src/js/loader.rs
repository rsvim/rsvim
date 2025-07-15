//! Js module loader.

use crate::js::module::{ModulePath, ModuleSource};
// use crate::js::transpiler::Jsx;
// use crate::js::transpiler::Wasm;
use crate::prelude::*;

use std::path::PathBuf;

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
/// 3. URL module loader, provide remote modules on network URI and file URI (file URI can be local
///    filesystem).
pub trait ModuleLoader {
  /// Resolve module path by its specifier.
  ///
  /// - For core module loader, it still returns the core module specifier (core modules don't have
  ///   a file path on local filesystem).
  /// - For fs module loader, it returns the full file path on local filesystem.
  /// - For url module loader, it returns remote URI (network URI or file URI) which indicates a
  ///   network location that can download the resource.
  ///
  /// NOTE: (To simplifies the architecture, ) all resolving process are synchronize, not
  /// asynchronize.
  fn resolve(
    &self,
    base: Option<&str>,
    runtime_paths: &Vec<PathBuf>,
    specifier: &str,
  ) -> AnyResult<ModulePath>;

  /// Load the module source by its module path ([`ModulePath`]).
  ///
  /// - For core module loader, it returns the core module source.
  /// - For fs module loader, it reads the file content on local filesystem and returns the module
  ///   source.
  /// - For url module loader, it first downloads the remote resource to local filesystem (and
  ///   caches them), then reads the local cache and returns the module source.
  ///
  /// NOTE: (To simplifies the architecture, ) all loading process are synchronize, not
  /// asynchronize. Even network downloading process is synchronize (at least for now). But in
  /// real-world, we need to provide a way to help user downloading and install the remote
  /// packages/plugins to local machine first, before they really start Rsvim editor, to avoid this
  /// issue.
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource>;
}
