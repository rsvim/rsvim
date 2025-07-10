//! Core module loader.

use crate::js::constant::WINDOWS_REGEX;
use crate::js::module::CORE_MODULES;
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
pub struct CoreModuleLoader {}

impl ModuleLoader for CoreModuleLoader {
  fn resolve(&self, _: Option<&str>, specifier: &str) -> AnyResult<ModulePath> {
    assert!(CORE_MODULES().contains_key(specifier));
    Ok(specifier.to_string())
  }
  fn load(&self, specifier: &str) -> AnyResult<ModuleSource> {
    // Since any errors will be caught at the resolve stage, we can
    // go ahead an unwrap the value with no worries.
    Ok(CORE_MODULES().get(specifier).unwrap().to_string())
  }
}
