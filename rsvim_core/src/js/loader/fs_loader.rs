//! Fs based module loader.

use crate::js::constant::WINDOWS_REGEX;
use crate::js::module::ModulePath;
use crate::js::module::ModuleSource;
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

#[derive(Default)]
pub struct FsModuleLoader {}

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
  /// Resolve specifier path on local file system.
  fn resolve(&self, base: Option<&str>, specifier: &str) -> AnyResult<ModulePath> {
    // Resolve absolute import.
    if specifier.starts_with('/') || WINDOWS_REGEX().is_match(specifier) {
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
