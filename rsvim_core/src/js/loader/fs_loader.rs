//! Fs (filesystem) module loader.

use crate::js::loader::{AsyncModuleLoader, ModuleLoader};
use crate::js::module::{ModulePath, ModuleSource};
use crate::js::transpiler::TypeScript;
// use crate::js::transpiler::Jsx;
// use crate::js::transpiler::Wasm;
use crate::prelude::*;

use async_trait::async_trait;
use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

const FILE_EXTENSIONS: &[&str] = &["js", "mjs", "ts", "json", "wasm"];
const PACKAGE_FILES: &[&str] = &["package.json", "package.json5"];

#[derive(Default)]
/// Fs (filesystem) module loader.
pub struct FsModuleLoader;

macro_rules! path_not_found1 {
  ($path:expr) => {
    anyhow::bail!(format!("Module path {:?} not found", $path))
  };
}

macro_rules! path_not_found2 {
  ($path:expr, $e:expr) => {
    anyhow::bail!(format!("Module path {:?} not found: {:?}", $path, $e))
  };
}

// Transforms `PathBuf` into `String`.
pub fn transform(path: PathBuf) -> String {
  path.into_os_string().into_string().unwrap()
}

/// Checks if path is a JSON file.
fn is_json_import(path: &Path) -> bool {
  path
    .extension()
    .map(|value| value == "json")
    .unwrap_or(false)
}

/// Wraps JSON data into an ES module (using v8's built in objects).
fn wrap_json(source: &str) -> String {
  format!("export default JSON.parse(`{source}`);")
}

mod sync_resolve {
  use super::*;

  pub fn resolve_file(path: &Path) -> AnyResult<ModulePath> {
    if path.is_file() {
      return Ok(transform(path.to_path_buf()));
    }

    if path.extension().is_none() {
      for ext in FILE_EXTENSIONS {
        let ext_path = path.with_extension(ext);
        if ext_path.is_file() {
          return Ok(transform(ext_path.to_path_buf()));
        }
      }
    }

    path_not_found1!(path)
  }

  macro_rules! _resolve_npm {
    ($field:expr,$path:expr) => {
      let json_path = $path.join(Path::new($field.as_str().unwrap()));
      if json_path.is_file() {
        return Ok(transform(json_path));
      }
    };
  }

  // Case-1: "exports" is plain string
  //
  // ```json
  // {
  //   "exports": "./index.js"
  // }
  // ```
  //
  // Case-2: "exports" is json object and use "." field
  //
  // ```json
  // {
  //   "exports": {
  //     ".": "./index.js"
  //   }
  // }
  // ```
  pub fn resolve_node_module(path: &Path) -> AnyResult<ModulePath> {
    if path.is_dir() {
      for pkg in PACKAGE_FILES {
        let pkg_path = path.join(pkg);
        if pkg_path.is_file() {
          match std::fs::read_to_string(pkg_path) {
            Ok(pkg_src) => {
              match serde_json::from_str::<serde_json::Value>(&pkg_src) {
                Ok(pkg_json) => {
                  for field in ["exports", "main"] {
                    match pkg_json.get(field) {
                      Some(json_entry) => {
                        if json_entry.is_string() {
                          _resolve_npm!(json_entry, path);
                        }

                        if json_entry.is_object() {
                          match json_entry.get(".") {
                            Some(json_entry_cwd) => {
                              if json_entry_cwd.is_string() {
                                _resolve_npm!(json_entry_cwd, path);
                              }
                            }
                            None => { /* do nothing */ }
                          }
                        }
                      }
                      None => { /* do nothing */ }
                    }
                  }
                }
                Err(e) => path_not_found2!(path, e),
              }
            }
            Err(e) => path_not_found2!(path, e),
          }
        }
      }

      // Fallback to default `index.js`
      for ext in FILE_EXTENSIONS {
        let path = path.join(format!("index.{ext}"));
        if path.is_file() {
          return Ok(transform(path));
        }
      }
    }

    path_not_found1!(path)
  }
}

mod sync_load {
  use super::*;

  /// Loads contents from a file.
  pub fn load_source(path: &Path) -> AnyResult<ModuleSource> {
    let source = std::fs::read_to_string(path)?;
    let source = if is_json_import(path) {
      wrap_json(source.as_str())
    } else {
      source
    };

    Ok(source)
  }

  /// Loads import as file.
  pub fn load_as_file(path: &Path) -> AnyResult<(PathBuf, ModuleSource)> {
    // If path is a file.
    if path.is_file() {
      return match load_source(path) {
        Ok(source) => Ok((path.to_path_buf(), source)),
        Err(e) => Err(e),
      };
    }

    path_not_found1!(path)
  }
}

mod async_load {
  use super::*;

  pub async fn async_load_source(path: &Path) -> AnyResult<ModuleSource> {
    let source = tokio::fs::read_to_string(path).await?;
    let source = if is_json_import(path) {
      wrap_json(source.as_str())
    } else {
      source
    };

    Ok(source)
  }

  pub async fn async_load_as_file(
    path: &Path,
  ) -> AnyResult<(PathBuf, ModuleSource)> {
    // If path is a file.
    if path.is_file() {
      return match async_load_source(path).await {
        Ok(source) => Ok((path.to_path_buf(), source)),
        Err(e) => Err(e),
      };
    }

    path_not_found1!(path)
  }
}

impl ModuleLoader for FsModuleLoader {
  /// Resolve module path by specifier in local filesystem.
  ///
  /// It tries to resolve npm packages, thus we can directly use npm registry to maintain plugins.
  /// But node/npm have quite a history, it requires quite an effort to be fully compatible with,
  /// we only choose to maintain a small subset (at least for now):
  ///
  /// 1. The "common js" standard is not supported.
  /// 2. The `cjs` file extension is not supported.
  /// 3. The `require` keyword is not supported.
  ///
  /// For more details about node/npm package, please see: <https://nodejs.org/api/packages.html>.
  fn resolve(
    &self,
    base: Option<&str>,
    specifier: &str,
  ) -> AnyResult<ModulePath> {
    // Full file path, start with '/' or 'C:\\'.
    if specifier.starts_with('/')
      || WINDOWS_DRIVE_BEGIN_REGEX.is_match(specifier)
    {
      let path = Path::new(specifier).absolutize()?.to_path_buf();
      return sync_resolve::resolve_file(path.as_path())
        .or_else(|_| sync_resolve::resolve_node_module(path.as_path()));
    }

    // Relative file path.
    if specifier.starts_with("./") || specifier.starts_with("../") {
      let base = match base {
        Some(value) => Path::new(value).parent().unwrap().to_path_buf(),
        None => path_not_found1!(specifier),
      };

      let path = base.join(specifier).absolutize()?.to_path_buf();
      return sync_resolve::resolve_file(path.as_path())
        .or_else(|_| sync_resolve::resolve_node_module(path.as_path()));
    }

    // Config home
    match PATH_CONFIG.config_home() {
      Some(config_home) => {
        // Simple path in config home directory `${config_home}`.
        let simple_path =
          config_home.join(specifier).absolutize()?.to_path_buf();
        // let simple_path = simple_path.absolutize()?;
        let maybe_path = sync_resolve::resolve_file(simple_path.as_path())
          .or_else(|_| {
            sync_resolve::resolve_node_module(simple_path.as_path())
          });
        if maybe_path.is_ok() {
          return maybe_path;
        }

        // Npm module path in `${config_home}/node_modules`.
        let npm_path = config_home.join("node_modules").join(specifier);
        let npm_path = npm_path.absolutize()?;
        sync_resolve::resolve_node_module(&npm_path)
      }
      None => path_not_found1!(specifier),
    }
  }

  /// Load module source by its module path, it can be either a file path, or a directory path.
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    // Load source.
    let path = Path::new(specifier);
    let maybe_source = sync_load::load_as_file(path);

    let (path, source) = match maybe_source {
      Ok((path, source)) => (path, source),
      Err(e) => return Err(e),
    };

    let path_extension = path.extension().unwrap().to_str().unwrap();
    let fname = path.to_str();

    // Use a preprocessor if necessary.
    match path_extension {
      // "wasm" => Ok(Wasm::parse(&source)),
      "ts" => TypeScript::compile(fname, &source),
      // "jsx" => {
      //   Jsx::compile(fname, &source).map_err(|e| JsRuntimeErr::Message(e.to_string()).into())
      // }
      // "tsx" => Jsx::compile(fname, &source)
      //   .and_then(|output| TypeScript::compile(fname, &output))
      //   .map_err(|e| JsRuntimeErr::Message(e.to_string()).into()),
      _ => Ok(source),
    }
  }
}

#[derive(Default)]
/// Async [`FsModuleLoader`].
///
/// NOTE: This is only allow to use in event loop, i.e. with tokio runtime, not
/// in js runtime.
pub struct AsyncFsModuleLoader;

#[async_trait]
impl AsyncModuleLoader for AsyncFsModuleLoader {
  async fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    // Load source.
    let path = Path::new(specifier);
    let maybe_source = async_load::async_load_as_file(path).await;

    let (path, source) = match maybe_source {
      Ok((path, source)) => (path, source),
      Err(e) => return Err(e),
    };

    let path_extension = path.extension().unwrap().to_str().unwrap();
    let fname = path.to_str();

    // Use a preprocessor if necessary.
    match path_extension {
      // "wasm" => Ok(Wasm::parse(&source)),
      "ts" => TypeScript::compile(fname, &source),
      // "jsx" => {
      //   Jsx::compile(fname, &source).map_err(|e| JsRuntimeErr::Message(e.to_string()).into())
      // }
      // "tsx" => Jsx::compile(fname, &source)
      //   .and_then(|output| TypeScript::compile(fname, &output))
      //   .map_err(|e| JsRuntimeErr::Message(e.to_string()).into()),
      _ => Ok(source),
    }
  }
}
