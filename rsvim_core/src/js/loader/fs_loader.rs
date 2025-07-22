//! Fs (filesystem) module loader.

use crate::constant::PathConfig;
use crate::js::loader::ModuleLoader;
use crate::js::module::{ModulePath, ModuleSource};
// use crate::js::transpiler::Jsx;
use crate::js::transpiler::TypeScript;
// use crate::js::transpiler::Wasm;
use crate::prelude::*;

// use regex::Regex;
// use sha::sha1::Sha1;
// use sha::utils::Digest;
// use sha::utils::DigestExt;
use path_absolutize::Absolutize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
// use url::Url;

static FILE_EXTENSIONS: &[&str] =
  &["js", "mjs", "jsx", "ts", "tsx", "json", "wasm"];

#[derive(Default)]
/// Fs (filesystem) module loader.
pub struct FsModuleLoader;

impl FsModuleLoader {
  // Transforms `PathBuf` into `String`.
  fn transform(&self, path: PathBuf) -> String {
    path.into_os_string().into_string().unwrap()
  }

  /// Checks if path is a JSON file.
  fn is_json_import(&self, path: &Path) -> bool {
    path
      .extension()
      .map(|value| value == "json")
      .unwrap_or(false)
  }

  /// Wraps JSON data into an ES module (using v8's built in objects).
  fn wrap_json(&self, source: &str) -> String {
    format!("export default JSON.parse(`{source}`);")
  }

  /// Loads contents from a file.
  fn load_source(&self, path: &Path) -> AnyResult<ModuleSource> {
    let source = fs::read_to_string(path)?;
    let source = if self.is_json_import(path) {
      self.wrap_json(source.as_str())
    } else {
      source
    };

    Ok(source)
  }

  /// Loads import as file.
  fn load_as_file(&self, path: &Path) -> AnyResult<ModuleSource> {
    // If path is a file.
    if path.is_file() {
      return self.load_source(path);
    }

    // If path is not a file, and it doesn't has a file extension, try to find it by adding the
    // file extension.
    if path.extension().is_none() {
      for ext in FILE_EXTENSIONS {
        let ext_path = path.with_extension(ext);
        if ext_path.is_file() {
          return self.load_source(&ext_path);
        }
      }
    }

    // 3. Bail out with an error.
    let path_display = path.display();
    anyhow::bail!(format!("Module path not found: {path_display:?}"));
  }

  /// Loads import as directory using the 'index.[ext]' convention.
  ///
  /// TODO: In the future, we may want to also support the npm package.
  fn load_as_directory(&self, path: &Path) -> AnyResult<ModuleSource> {
    for ext in FILE_EXTENSIONS {
      let path = &path.join(format!("index.{ext}"));
      if path.is_file() {
        return self.load_source(path);
      }
    }
    let path_display = path.display();
    anyhow::bail!(format!("Module path not found: {path_display:?}"));
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
    path_cfg: &PathConfig,
  ) -> AnyResult<ModulePath> {
    // Full file path, start with '/' or 'C:\\'.
    if specifier.starts_with('/')
      || WINDOWS_DRIVE_BEGIN_REGEX.is_match(specifier)
    {
      return Ok(
        self.transform(Path::new(specifier).absolutize()?.to_path_buf()),
      );
    }

    // Relative file path.
    if specifier.starts_with("./") || specifier.starts_with("../") {
      let base = match base {
        Some(value) => Path::new(value).parent().unwrap().to_path_buf(),
        None => {
          anyhow::bail!(format!("Module specifier not found: {specifier:?}"))
        }
      };

      return Ok(
        self.transform(base.join(specifier).absolutize()?.to_path_buf()),
      );
    }

    // For other
    match path_cfg.config_home() {
      Some(config_home) => {
        // Simple file path in config home directory `${config_home}`.
        let simple_specifier = config_home.join(specifier);
        match simple_specifier.absolutize() {
          Ok(simple_path) => {
            if simple_path.exists() {
              return Ok(self.transform(simple_path.to_path_buf()));
            }
          }
          Err(e) => anyhow::bail!(format!(
            "Module specifier error: {specifier:?}, {e:?}"
          )),
        }

        // Npm file path in `${config_home}/node_modules`.
        let npm_specifier = config_home.join("node_modules").join(specifier);
        match npm_specifier.absolutize() {
          Ok(npm_path) => {
            if npm_path.exists() {
              return Ok(self.transform(npm_path.to_path_buf()));
            }
          }
          Err(e) => anyhow::bail!(format!(
            "Module specifier error: {specifier:?}, {e:?}"
          )),
        }

        // Otherwise we try to resolve it as node/npm package.
        anyhow::bail!(format!("Module specifier not found: {specifier:?}"));
      }
      None => {
        anyhow::bail!(format!("Module specifier not found: {specifier:?}"));
      }
    }
  }

  /// Load module source by its module path, it can be either a file path, or a directory path.
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    // Load source.
    let path = Path::new(specifier);
    let maybe_source = self
      .load_as_file(path)
      .or_else(|_| self.load_as_directory(path));

    // Append default extension (if none specified).
    let path = match path.extension() {
      Some(_) => path.into(),
      None => path.with_extension("js"),
    };

    let source = match maybe_source {
      Ok(source) => source,
      Err(_) => {
        anyhow::bail!(format!("Module path not found \"{}\"", path.display()))
      }
    };

    let path_extension = path.extension().unwrap().to_str().unwrap();
    let fname = path.to_str();

    // Use a preprocessor if necessary.
    match path_extension {
      // "wasm" => Ok(Wasm::parse(&source)),
      "ts" => TypeScript::compile(fname, &source)
        .map_err(|e| JsRuntimeErr::Message(e.to_string()).into()),
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
