//! Js extension transpiler.

use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::anyhow;
use deno_core::error::AnyError;
use deno_core::FastString;
use deno_core::ModuleSpecifier;
use deno_core::SourceMapData;
use std::borrow::Cow;
use std::env::current_dir;
use std::path::Path;

pub type ModuleContents = (String, Option<SourceMapData>);

fn should_transpile(media_type: MediaType) -> bool {
  matches!(
    media_type,
    MediaType::Jsx
      | MediaType::TypeScript
      | MediaType::Mts
      | MediaType::Cts
      | MediaType::Dts
      | MediaType::Dmts
      | MediaType::Dcts
      | MediaType::Tsx
  )
}

/// Transpiles source code from TS to JS without typechecking
pub fn transpile(
  module_specifier: &ModuleSpecifier,
  code: &str,
) -> Result<ModuleContents, anyhow::Error> {
  let media_type = MediaType::from_specifier(module_specifier);
  let should_transpile = should_transpile(media_type);

  let code = if should_transpile {
    let sti = SourceTextInfo::from_string(code.to_string());
    let text = sti.text();
    let parsed = deno_ast::parse_module(ParseParams {
      specifier: module_specifier.clone(),
      text,
      media_type,
      capture_tokens: false,
      scope_analysis: false,
      maybe_syntax: None,
    })?;

    let transpile_options = deno_ast::TranspileOptions::default();
    let emit_options = deno_ast::EmitOptions {
      remove_comments: false,
      source_map: deno_ast::SourceMapOption::Separate,
      inline_sources: false,
      ..Default::default()
    };
    let res = parsed
      .transpile(&transpile_options, &emit_options)?
      .into_source();

    let text = res.source;
    // Convert utf8 bytes to a string
    let text = String::from_utf8(text)?;
    let source_map: Option<SourceMapData> = res.source_map.map(Into::into);
    (text, source_map)
  } else {
    (code.to_string(), None)
  };

  Ok(code)
}

fn resolve_path(
  path_str: impl AsRef<Path>,
  current_dir: &Path,
) -> Result<ModuleSpecifier, deno_core::ModuleResolutionError> {
  let path = current_dir.join(path_str);
  let path = deno_core::normalize_path(path);
  deno_core::url::Url::from_file_path(&path)
    .map_err(|()| deno_core::ModuleResolutionError::InvalidPath(path))
}

fn to_module_specifier(s: &str, base: Option<&Path>) -> Result<ModuleSpecifier, anyhow::Error> {
  let path = match base {
    Some(base) => resolve_path(s, base),
    None => resolve_path(s, &current_dir()?),
  }?;
  Ok(path)
}

/// Transpile an extension
#[allow(clippy::type_complexity)]
pub fn transpile_extension(
  specifier: &FastString,
  code: &FastString,
) -> Result<(FastString, Option<Cow<'static, [u8]>>), AnyError> {
  // Get the ModuleSpecifier from the FastString
  let specifier = specifier.as_str();
  let specifier = to_module_specifier(specifier, None)?;
  let code = code.as_str();

  let (code, source_map) = transpile(&specifier, code)?;
  let code = FastString::from(code);

  Ok((code, source_map))
}
