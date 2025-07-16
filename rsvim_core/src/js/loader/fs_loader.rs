//! Fs (filesystem) module loader.

use crate::js::loader::ModuleLoader;
use crate::js::module::{ModulePath, ModuleSource};
// use crate::js::transpiler::Jsx;
use crate::js::transpiler::TypeScript;
// use crate::js::transpiler::Wasm;
use crate::prelude::*;

use anyhow::bail;
// use regex::Regex;
// use sha::sha1::Sha1;
// use sha::utils::Digest;
// use sha::utils::DigestExt;
use path_absolutize::Absolutize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
// use url::Url;

static FILE_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "json", "json5", "wasm"];

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
    bail!(format!("Module not found \"{}\"", path.display()));
  }

  /// Loads import as directory using the 'index.[ext]' convention.
  fn load_as_directory(&self, path: &Path) -> AnyResult<ModuleSource> {
    for ext in FILE_EXTENSIONS {
      let path = &path.join(format!("index.{ext}"));
      if path.is_file() {
        return self.load_source(path);
      }
    }
    bail!(format!("Module not found \"{}\"", path.display()));
  }
}

impl ModuleLoader for FsModuleLoader {
  /// Resolve module path by specifier in local filesystem.
  ///
  /// It tries to resolve npm packages, thus we can directly use npm registry to maintain plugins.
  /// But node/npm have quite a history, it requires quite an effort to be fully compatible with,
  /// we only choose to maintain a small subset (at least for now):
  ///
  /// 1. The "common js" standard is not implemented.
  /// 2. All `cjs`/`mjs`/`js` are recognized as ES module, not common js.
  /// 3. The `require` keyword is not supported.
  ///
  /// There are several use cases in module resolving process.
  ///
  /// # Full file path with file extension
  ///
  /// For example:
  ///
  /// ```javascript
  /// import syntaxes from "/home/usr/rsvim/.rsvim/syntaxes.js";
  /// ```
  ///
  /// The specifier is the same with its module path.
  ///
  /// # Relative file path with file extension
  ///
  /// For example:
  ///
  /// ```javascript
  /// import syntaxes from "./syntaxes.js";
  /// import syntaxes from "../utils/syntaxes.js";
  /// ```
  ///
  /// The module path is resolved based on **current** module's file path. For example we have
  /// below module structure:
  ///
  /// ```text
  /// ~/.rsvim/
  /// |- index.js
  /// |- syntaxes.js  -> `syntaxes1`
  /// |- util/
  ///    |- syntaxes.js -> `syntaxes2`
  /// ```
  ///
  /// In `index.js`:
  ///
  /// ```javascript
  /// import syntaxes1 from "./syntaxes.js";
  /// import syntaxes2 from "./util/syntaxes.js";
  /// ```
  ///
  /// NOTE: This also works for node/npm package.
  ///
  /// # File name with file extension
  ///
  /// For example:
  ///
  /// ```javascript
  /// import syntaxes from "syntaxes.js";
  /// ```
  ///
  /// The specifier `"syntaxes.js"` is not full file path nor relative file path. Rsvim will search
  /// it in config home (`$XDG_CONFIG_HOME/rsvim` or `$HOME/.rsvim`), (let's say the config home is
  /// `${rsvim_config_home}`) the module path is `${rsvim_config_home}/syntaxes.js`.
  ///
  /// # Node/npm package without file extension
  ///
  /// Rsvim tries to resolve node packages, thus we can directly use npm's registry to publish
  /// Rsvim plugins and even manage them with the `npm` executable. But node/npm packages have
  /// quite a history, it requires a lot of effort to be fully compatible with it, here we only
  /// implement part of the ES modules (at least for now):
  ///
  /// 1.
  ///
  /// For more details about node/npm package, please see: <https://nodejs.org/api/packages.html>.
  fn resolve(&self, base: Option<&str>, specifier: &str) -> AnyResult<ModulePath> {
    // Resolve absolute import.
    if specifier.starts_with('/') || WINDOWS_DRIVE_BEGIN_REGEX.is_match(specifier) {
      return Ok(self.transform(Path::new(specifier).absolutize()?.to_path_buf()));
    }

    // Resolve relative import.
    // FIXME: Here we should always disable CWD as a parent path to resolve modules.
    // Because for rsvim editor, the modules are stored in user config directories, not CWD.
    // CWD is mostly for general runtimes such as node/deno project.
    let cwd = &std::env::current_dir().unwrap();
    let base = base.map(|v| Path::new(v).parent().unwrap()).unwrap_or(cwd);

    if specifier.starts_with("./") || specifier.starts_with("../") {
      return Ok(self.transform(base.join(specifier).absolutize()?.to_path_buf()));
    }

    bail!(format!("Module not found \"{specifier}\""));
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
      Err(_) => bail!(format!("Module not found \"{}\"", path.display())),
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
