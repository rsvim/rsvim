//! Js module.

use crate::js::JsRuntime;
use crate::js::loader::{CoreModuleLoader, FsModuleLoader, ModuleLoader};
use crate::prelude::*;

use std::sync::OnceLock;
use tracing::trace;

// Re-export
pub use es_module::*;
pub use import_map::*;
pub use module_graph::*;
pub use module_map::*;

pub mod es_module;
pub mod import_map;
pub mod module_graph;
pub mod module_map;

/// Module path on local file system.
pub type ModulePath = String;

/// Module source code.
pub type ModuleSource = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Module import status.
///
/// NOTE: All modules (plugins/packages) will be local files on user's operating system, no
/// network/http modules will be fetching. The only one use case of `Resolving` status should be
/// dynamically import and its `Promise`.
pub enum ModuleStatus {
  // Indicates the module **itself** is being fetched.
  Fetching,
  // Indicates the module dependencies are being fetched.
  Resolving,
  // Indicates the module has ben seen before.
  Duplicate,
  // Indicates the module (include its dependencies) is resolved.
  Ready,
}

#[allow(non_snake_case)]
pub fn CORE_MODULES() -> &'static HashMap<&'static str, &'static str> {
  static VALUE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
  VALUE.get_or_init(|| {
    let modules = vec![
      // ("rsvim:ext/infra", include_str!("./runtime/00__infra.js")),
      // ("console", include_str!("./js/console.js")),
      // ("events", include_str!("./js/events.js")),
      // ("process", include_str!("./js/process.js")),
      // ("timers", include_str!("./js/timers.js")),
      // ("assert", include_str!("./js/assert.js")),
      // ("util", include_str!("./js/util.js")),
      // ("fs", include_str!("./module/fs.js")),
      // ("perf_hooks", include_str!("./js/perf-hooks.js")),
      // ("colors", include_str!("./js/colors.js")),
      // ("dns", include_str!("./js/dns.js")),
      // ("net", include_str!("./js/net.js")),
      // ("test", include_str!("./js/test.js")),
      // ("stream", include_str!("./js/stream.js")),
      // ("http", include_str!("./js/http.js")),
      // ("@web/abort", include_str!("./js/abort-controller.js")),
      // ("@web/text_encoding", include_str!("./js/text-encoding.js")),
      // ("@web/clone", include_str!("./js/structured-clone.js")),
      // ("@web/fetch", include_str!("./js/fetch.js")),
    ];
    HashMap::from_iter(modules)
  })
}

/// Create v8 script origin.
pub fn create_origin<'s>(
  scope: &mut v8::HandleScope<'s, ()>,
  name: &str,
  is_module: bool,
) -> v8::ScriptOrigin<'s> {
  let name = v8::String::new(scope, name).unwrap();
  let source_map = v8::undefined(scope);

  v8::ScriptOrigin::new(
    scope,
    name.into(),
    0,
    0,
    false,
    0,
    Some(source_map.into()),
    false,
    false,
    is_module,
    None,
  )
}

/// Resolves an import using the appropriate loader.
/// Returns full path on local file system.
pub fn resolve_import(
  base: Option<&str>,
  specifier: &str,
  ignore_core_modules: bool,
  import_map: Option<ImportMap>,
) -> AnyResult<ModulePath> {
  // Use import-maps if available.
  // FIXME: This is not supported now.
  debug_assert!(import_map.is_none());
  let specifier = match import_map {
    Some(map) => map.lookup(specifier).unwrap_or_else(|| specifier.into()),
    None => specifier.into(),
  };

  // Look the params and choose a loader, then resolve module.
  let is_core_module_import = CORE_MODULES().contains_key(specifier.as_str());
  if is_core_module_import && !ignore_core_modules {
    CoreModuleLoader {}.resolve(base, &specifier)
  } else {
    FsModuleLoader {}.resolve(base, &specifier)
  }
}

/// Loads an import using the appropriate loader.
pub fn load_import(specifier: &str, _skip_cache: bool) -> AnyResult<ModuleSource> {
  // // Look the params and choose a loader.
  // let loader: Box<dyn ModuleLoader> = match (
  //   CORE_MODULES().contains_key(specifier),
  //   WINDOWS_REGEX().is_match(specifier),
  //   Url::parse(specifier).is_ok(),
  // ) {
  //   (true, _, _) => Box::new(CoreModuleLoader),
  //   (_, true, _) => Box::new(FsModuleLoader),
  //   (_, _, true) => Box::new(UrlModuleLoader { skip_cache }),
  //   _ => Box::new(FsModuleLoader),
  // };
  //
  // // Load module.
  // loader.load(specifier)

  let is_core_module_import = CORE_MODULES().contains_key(specifier);
  if is_core_module_import {
    CoreModuleLoader {}.load(specifier)
  } else {
    FsModuleLoader {}.load(specifier)
  }

  // // We don't actually have core modules
  // FsModuleLoader {}.load(specifier)
}

/// NOTE: Not support for now.
pub async fn load_import_async(specifier: &str, skip_cache: bool) -> AnyResult<ModuleSource> {
  load_import(specifier, skip_cache)
}

/// Resolves static import, synchronously.
/// See: <https://source.chromium.org/chromium/v8/v8.git/+/51e736ca62bd5c7bfd82488a5587fed31dbf45d5:src/d8.cc;l=741>.
pub fn fetch_module_tree<'a>(
  scope: &mut v8::HandleScope<'a>,
  filename: &str,
  source: Option<&str>,
) -> Option<v8::Local<'a, v8::Module>> {
  // Create a script origin.
  let origin = create_origin(scope, filename, true);
  let state = JsRuntime::state(scope);

  // Find appropriate loader if source is empty.
  let source = match source {
    Some(source) => source.into(),
    None => load_import(filename, true).unwrap(),
  };
  trace!(
    "Loaded main js module filename: {:?}, source: {:?}",
    filename,
    if source.as_str().len() > 20 {
      String::from(&source.as_str()[..20]) + "..."
    } else {
      String::from(source.as_str())
    }
  );
  let source = v8::String::new(scope, &source).unwrap();
  let mut source = v8::script_compiler::Source::new(source, Some(&origin));

  let module = v8::script_compiler::compile_module(scope, &mut source)?;

  // Subscribe module to the module-map.
  let module_ref = v8::Global::new(scope, module);
  state.borrow_mut().module_map.insert(filename, module_ref);

  let requests = module.get_module_requests();
  trace!("Get {} module requests", requests.length());

  for i in 0..requests.length() {
    // Get import request from the `module_requests` array.
    let request = requests.get(scope, i).unwrap();
    let request = v8::Local::<v8::ModuleRequest>::try_from(request).unwrap();

    // Transform v8's ModuleRequest into Rust string.
    let specifier = request.get_specifier().to_rust_string_lossy(scope);
    let specifier = resolve_import(Some(filename), &specifier, false, None).unwrap();
    trace!(
      "Resolved dependency js module base: {:?}, specifier: {:?}",
      filename,
      specifier.as_str(),
    );

    // Resolve subtree of modules.
    if !state.borrow().module_map.index.contains_key(&specifier) {
      fetch_module_tree(scope, &specifier, None)?;
    }
  }

  Some(module)
}
