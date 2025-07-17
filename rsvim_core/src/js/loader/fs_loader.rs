//! Fs (filesystem) module loader.

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

static FILE_EXTENSIONS: &[&str] = &[
  "js", "mjs", "cjs", "jsx", "ts", "tsx", "json", "json5", "wasm",
];

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
    match path.extension() {
      Some(value) => value == "json",
      None => false,
    }
  }

  /// Wraps JSON data into an ES module (using v8's built in objects).
  fn wrap_json(&self, source: &str) -> String {
    format!("export default JSON.parse(`{source}`);")
  }

  /// Loads contents from a file.
  fn load_source(&self, path: &Path) -> AnyResult<ModuleSource> {
    let source = fs::read_to_string(path)?;
    let source = match self.is_json_import(path) {
      true => self.wrap_json(source.as_str()),
      false => source,
    };

    Ok(source)
  }

  /// Loads import as file.
  fn load_as_file(&self, path: &Path) -> AnyResult<ModuleSource> {
    // 1. Check if path is already a valid file.
    if path.is_file() {
      return self.load_source(path);
    }

    // 2. Check if we need to add an extension.
    if path.extension().is_none() {
      for ext in FILE_EXTENSIONS {
        let path = &path.with_extension(ext);
        if path.is_file() {
          return self.load_source(path);
        }
      }
    }

    // 3. Bail out with an error.
    let path_display = path.display();
    anyhow::bail!(format!("Module path not found: {path_display:?}"));
  }

  /// Loads import as directory using the 'index.[ext]' convention.
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
  /// 2. All `cjs`/`mjs`/`js` are recognized as ES module, not common js.
  /// 3. The `require` keyword is not supported.
  ///
  /// For more details about node/npm package, please see: <https://nodejs.org/api/packages.html>.
  fn resolve(&self, base: Option<&str>, specifier: &str) -> AnyResult<ModulePath> {
    // Full file path, start with '/' or 'C:\\'.
    if specifier.starts_with('/') || WINDOWS_DRIVE_BEGIN_REGEX.is_match(specifier) {
      return Ok(self.transform(Path::new(specifier).absolutize()?.to_path_buf()));
    }

    // Resolve file path.
    let base = match base {
      Some(value) => Path::new(value).parent().unwrap().to_path_buf(),
      None => match &*CONFIG_HOME_PATH {
        Some(config_home) => config_home.to_path_buf(),
        None => {
          anyhow::bail!(format!("Module specifier not found: {specifier:?}"));
        }
      },
    };

    if specifier.starts_with("./") || specifier.starts_with("../") {
      return Ok(self.transform(base.join(specifier).absolutize()?.to_path_buf()));
    }

    anyhow::bail!(format!("Module specifier not found: {specifier:?}"));
  }

  /// Load module source by its module path (full file path).
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
      Err(_) => anyhow::bail!(format!("Module path not found \"{}\"", path.display())),
    };

    let path_extension = path.extension().unwrap().to_str().unwrap();
    let fname = path.to_str();

    // Use a preprocessor if necessary.
    match path_extension {
      // "wasm" => Ok(Wasm::parse(&source)),
      "ts" => {
        TypeScript::compile(fname, &source).map_err(|e| JsRuntimeErr::Message(e.to_string()).into())
      }
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
