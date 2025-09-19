//! Fs (filesystem) module loader.

use crate::js::loader::AsyncModuleLoader;
use crate::js::loader::ModuleLoader;
use crate::js::module::ModulePath;
use crate::js::module::ModuleSource;
use crate::js::transpiler::TypeScript;
// use crate::js::transpiler::Jsx;
use crate::js::transpiler::Wasm;
use crate::prelude::*;
use async_trait::async_trait;
use oxc_resolver::ResolveOptions;
use oxc_resolver::Resolver;

macro_rules! path_not_found {
  ($path:expr) => {
    anyhow::bail!(format!("Module path NotFound({:?})", $path))
  };
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

    path_not_found!(path)
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

    path_not_found!(path)
  }
}

#[derive(Default)]
/// Fs (filesystem) module loader.
pub struct FsModuleLoader {
  resolver: Resolver,
}

impl FsModuleLoader {
  pub fn new() -> Self {
    let opts = ResolveOptions {
      extensions: vec![
        ".js".into(),
        ".ts".into(),
        ".mjs".into(),
        ".json".into(),
        ".wasm".into(),
      ],
      extension_alias: vec![
        (".js".into(), vec![".js".into()]),
        (".mjs".into(), vec![".mjs".into()]),
        (".ts".into(), vec![".ts".into()]),
        (".json".into(), vec![".json".into()]),
        (".wasm".into(), vec![".wasm".into()]),
      ],
      // builtin_modules: false,
      ..ResolveOptions::default()
    };
    Self {
      resolver: Resolver::new(opts),
    }
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
  fn resolve(&self, base: &str, specifier: &str) -> AnyResult<ModulePath> {
    let base = Path::new(base).to_path_buf();
    trace!(
      "|FsModuleLoader::resolve| base:{:?}, specifier:{:?}",
      base, specifier
    );
    match self.resolver.resolve(&base, specifier) {
      Ok(resolution) => Ok(resolution.path().to_string_lossy().to_string()),
      Err(e) => {
        let node_modules_home = base.join("node_modules");
        if node_modules_home.is_dir() {
          match self.resolver.resolve(node_modules_home, specifier) {
            Ok(resolution) => {
              Ok(resolution.path().to_string_lossy().to_string())
            }
            Err(e) => anyhow::bail!(format!("Module path {:?}", e)),
          }
        } else {
          anyhow::bail!(format!("Module path {:?}", e));
        }
      }
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

    let path_extension =
      path.extension().unwrap().to_string_lossy().to_string();
    let fname = path.to_str();

    // Use a preprocessor if necessary.
    match path_extension.as_str() {
      "wasm" => Ok(Wasm::parse(&source)),
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

    let path_extension =
      path.extension().unwrap().to_string_lossy().to_string();
    let fname = path.to_str();

    // Use a preprocessor if necessary.
    match path_extension.as_str() {
      "wasm" => Ok(Wasm::parse(&source)),
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
