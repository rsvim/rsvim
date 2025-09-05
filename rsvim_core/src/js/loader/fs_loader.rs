//! Fs (filesystem) module loader.

use crate::js::loader::{AsyncModuleLoader, ModuleLoader};
use crate::js::module::{ModulePath, ModuleSource};
// use crate::js::transpiler::Jsx;
use crate::js::transpiler::TypeScript;
// use crate::js::transpiler::Wasm;
use crate::prelude::*;

// use regex::Regex;
// use sha::sha1::Sha1;
// use sha::utils::Digest;
// use sha::utils::DigestExt;
use async_trait::async_trait;
use path_absolutize::Absolutize;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
// use url::Url;

static FILE_EXTENSIONS: &[&str] =
  &["js", "mjs", "jsx", "ts", "tsx", "json", "wasm"];

#[derive(Default)]
/// Fs (filesystem) module loader.
pub struct FsModuleLoader;

fn path_not_found<P>(path: P) -> String
where
  P: Into<OsString> + std::fmt::Debug,
{
  format!("Error: Module path {path:?} not found!")
}

fn path_not_found2<P>(path: P, e: std::io::Error) -> String
where
  P: Into<OsString> + std::fmt::Debug,
{
  format!("Error: Module path {path:?} not found: {e:?}")
}

// Transforms `PathBuf` into `String`.
fn transform(path: PathBuf) -> String {
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

/// Loads contents from a file.
fn load_source(path: &Path) -> AnyResult<ModuleSource> {
  let source = std::fs::read_to_string(path)?;
  let source = if is_json_import(path) {
    wrap_json(source.as_str())
  } else {
    source
  };

  Ok(source)
}

/// Async [`load_source`].
async fn async_load_source(path: &Path) -> AnyResult<ModuleSource> {
  let source = tokio::fs::read_to_string(path).await?;
  let source = if is_json_import(path) {
    wrap_json(source.as_str())
  } else {
    source
  };

  Ok(source)
}

/// Loads import as file.
fn load_as_file(path: &Path) -> AnyResult<(PathBuf, ModuleSource)> {
  // If path is a file.
  if path.is_file() {
    return match load_source(path) {
      Ok(source) => Ok((path.to_path_buf(), source)),
      Err(e) => Err(e),
    };
  }

  // If path is not a file, and it doesn't has a file extension, try to find it by adding the
  // file extension.
  if path.extension().is_none() {
    for ext in FILE_EXTENSIONS {
      let ext_path = path.with_extension(ext);
      if ext_path.is_file() {
        return match load_source(&ext_path) {
          Ok(source) => Ok((ext_path.to_path_buf(), source)),
          Err(e) => Err(e),
        };
      }
    }
  }

  // 3. Bail out with an error.
  anyhow::bail!(path_not_found(path));
}

/// Async [`load_as_file`].
async fn async_load_as_file(path: &Path) -> AnyResult<(PathBuf, ModuleSource)> {
  // If path is a file.
  if path.is_file() {
    return match async_load_source(path).await {
      Ok(source) => Ok((path.to_path_buf(), source)),
      Err(e) => Err(e),
    };
  }

  // If path is not a file, and it doesn't has a file extension, try to find it by adding the
  // file extension.
  if path.extension().is_none() {
    for ext in FILE_EXTENSIONS {
      let ext_path = path.with_extension(ext);
      if ext_path.is_file() {
        return match async_load_source(&ext_path).await {
          Ok(source) => Ok((ext_path.to_path_buf(), source)),
          Err(e) => Err(e),
        };
      }
    }
  }

  // 3. Bail out with an error.
  anyhow::bail!(path_not_found(path));
}

/// Loads import as directory using the 'index.[ext]' convention.
///
/// TODO: In the future, we may want to also support the npm package.
fn load_as_directory(path: &Path) -> AnyResult<(PathBuf, ModuleSource)> {
  for ext in FILE_EXTENSIONS {
    let path = &path.join(format!("index.{ext}"));
    if path.is_file() {
      return match load_source(path) {
        Ok(source) => Ok((path.to_path_buf(), source)),
        Err(e) => Err(e),
      };
    }
  }

  anyhow::bail!(path_not_found(path));
}

/// Async [`load_as_directory`].
async fn async_load_as_directory(
  path: &Path,
) -> AnyResult<(PathBuf, ModuleSource)> {
  for ext in FILE_EXTENSIONS {
    let path = &path.join(format!("index.{ext}"));
    if path.is_file() {
      return match async_load_source(path).await {
        Ok(source) => Ok((path.to_path_buf(), source)),
        Err(e) => Err(e),
      };
    }
  }

  anyhow::bail!(path_not_found(path));
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
      return Ok(transform(Path::new(specifier).absolutize()?.to_path_buf()));
    }

    // Relative file path.
    if specifier.starts_with("./") || specifier.starts_with("../") {
      let base = match base {
        Some(value) => Path::new(value).parent().unwrap().to_path_buf(),
        None => {
          anyhow::bail!(path_not_found(specifier))
        }
      };

      return Ok(transform(base.join(specifier).absolutize()?.to_path_buf()));
    }

    // For other
    match PATH_CONFIG.config_home() {
      Some(config_home) => {
        // Simple file path in config home directory `${config_home}`.
        let simple_specifier = config_home.join(specifier);
        match simple_specifier.absolutize() {
          Ok(simple_path) => {
            if simple_path.exists() {
              return Ok(transform(simple_path.to_path_buf()));
            }
          }
          Err(e) => {
            anyhow::bail!(path_not_found2(specifier, e))
          }
        }

        // Npm file path in `${config_home}/node_modules`.
        let npm_specifier = config_home.join("node_modules").join(specifier);
        match npm_specifier.absolutize() {
          Ok(npm_path) => {
            if npm_path.exists() {
              return Ok(transform(npm_path.to_path_buf()));
            }
          }
          Err(e) => {
            anyhow::bail!(path_not_found2(specifier, e))
          }
        }

        // Otherwise we try to resolve it as node/npm package.
        anyhow::bail!(path_not_found(specifier));
      }
      None => {
        anyhow::bail!(path_not_found(specifier));
      }
    }
  }

  /// Load module source by its module path, it can be either a file path, or a directory path.
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    // Load source.
    let path = Path::new(specifier);
    let maybe_source = load_as_file(path).or_else(|_| load_as_directory(path));

    let (path, source) = match maybe_source {
      Ok((path, source)) => (path, source),
      Err(_) => {
        anyhow::bail!(path_not_found(path))
      }
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
/// NOTE: This module loader is actually used in event loop, i.e. with tokio
/// runtime, not in js runtime.
pub struct AsyncFsModuleLoader;

#[async_trait]
impl AsyncModuleLoader for AsyncFsModuleLoader {
  async fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    // Load source.
    let path = Path::new(specifier);
    let maybe_source = match async_load_as_file(path).await {
      Ok(source) => Ok(source),
      Err(_) => async_load_as_directory(path).await,
    };

    let (path, source) = match maybe_source {
      Ok((path, source)) => (path, source),
      Err(_) => {
        anyhow::bail!(path_not_found(path))
      }
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
