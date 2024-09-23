//! Js modules.

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::env;
use std::path::Path;
use std::rc::Rc;

// pub mod transpiler;

#[derive(Debug, Clone)]
pub enum ImportKind {
  // Loading static imports.
  Static,
  // Loading a dynamic import.
  Dynamic(v8::Global<v8::PromiseResolver>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
  // Indicates the module is being fetched.
  Fetching,
  // Indicates the dependencies are being fetched.
  Resolving,
  // Indicates the module has ben seen before.
  Duplicate,
  // Indicates the modules is resolved.
  Ready,
}

#[derive(Debug)]
pub struct EsModule {
  pub path: ModulePath,
  pub status: ModuleStatus,
  pub dependencies: Vec<Rc<RefCell<EsModule>>>,
  pub exception: Rc<RefCell<Option<String>>>,
  pub is_dynamic_import: bool,
}

#[derive(Debug)]
pub struct ModuleGraph {
  pub kind: ImportKind,
  pub root_rc: Rc<RefCell<EsModule>>,
  pub same_origin: LinkedList<v8::Global<v8::PromiseResolver>>,
}

impl ModuleGraph {
  // Initializes a new graph resolving a static import.
  pub fn static_import(path: &str) -> ModuleGraph {
    // Create an ES module instance.
    let module = Rc::new(RefCell::new(EsModule {
      path: path.into(),
      status: ModuleStatus::Fetching,
      dependencies: vec![],
      exception: Rc::new(RefCell::new(None)),
      is_dynamic_import: false,
    }));

    Self {
      kind: ImportKind::Static,
      root_rc: module,
      same_origin: LinkedList::new(),
    }
  }

  // Initializes a new graph resolving a dynamic import.
  pub fn dynamic_import(path: &str, promise: v8::Global<v8::PromiseResolver>) -> ModuleGraph {
    // Create an ES module instance.
    let module = Rc::new(RefCell::new(EsModule {
      path: path.into(),
      status: ModuleStatus::Fetching,
      dependencies: vec![],
      exception: Rc::new(RefCell::new(None)),
      is_dynamic_import: true,
    }));

    Self {
      kind: ImportKind::Dynamic(promise),
      root_rc: module,
      same_origin: LinkedList::new(),
    }
  }
}

pub type ModulePath = String;
pub type ModuleSource = String;

pub struct ModuleMap {
  pub main: Option<ModulePath>,
  pub index: HashMap<ModulePath, v8::Global<v8::Module>>,
  pub seen: HashMap<ModulePath, ModuleStatus>,
  pub pending: Vec<Rc<RefCell<ModuleGraph>>>,
}

/// A single import mapping (specifier, target).
type ImportMapEntry = (String, String);

/// Key-Value entries representing WICG import-maps.
#[derive(Debug, Clone)]
pub struct ImportMap {
  map: Vec<ImportMapEntry>,
}

impl ImportMap {
  /// Creates an ImportMap from JSON text.
  pub fn parse_from_json(text: &str) -> anyhow::Result<ImportMap> {
    // Parse JSON string into serde value.
    let json: serde_json::Value = serde_json::from_str(text)?;
    let imports = json["imports"].to_owned();

    if imports.is_null() || !imports.is_object() {
      return Err(anyhow::anyhow!("Import map's 'imports' must be an object"));
    }

    let map: HashMap<String, String> = serde_json::from_value(imports)?;
    let mut map: Vec<ImportMapEntry> = Vec::from_iter(map);

    // Note: We're sorting the imports because we need to support "Packages"
    // via trailing slashes, so the lengthier mapping should always be selected.
    //
    // https://github.com/WICG/import-maps#packages-via-trailing-slashes

    map.sort_by(|a, b| b.0.cmp(&a.0));

    Ok(ImportMap { map })
  }

  /// Tries to match a specifier against an import-map entry.
  pub fn lookup(&self, specifier: &str) -> Option<String> {
    // Find a mapping if exists.
    let (base, mut target) = match self.map.iter().find(|(k, _)| specifier.starts_with(k)) {
      Some(mapping) => mapping.to_owned(),
      None => return None,
    };

    // The following code treats "./" as an alias for the CWD.
    if target.starts_with("./") {
      let cwd = env::current_dir().unwrap().to_string_lossy().to_string();
      target = target.replacen('.', &cwd, 1);
    }

    // Note: The reason we need this additional check below with the specifier's
    // extension (if exists) is to be able to support extension-less imports.
    //
    // https://github.com/WICG/import-maps#extension-less-imports

    match Path::new(specifier).extension() {
      Some(ext) => match Path::new(specifier) == Path::new(&base).with_extension(ext) {
        false => Some(specifier.replacen(&base, &target, 1)),
        _ => None,
      },
      None => Some(specifier.replacen(&base, &target, 1)),
    }
  }
}

/// Resolves an import using the appropriate loader.
pub fn resolve_import(
  base: Option<&str>,
  specifier: &str,
  ignore_core_modules: bool,
  import_map: Option<ImportMap>,
) -> anyhow::Result<ModulePath> {
  // Use import-maps if available.
  let specifier = match import_map {
    Some(map) => map.lookup(specifier).unwrap_or_else(|| specifier.into()),
    None => specifier.into(),
  };

  // Look the params and choose a loader.
  let loader: Box<dyn ModuleLoader> = {
    let is_core_module_import = CORE_MODULES.contains_key(specifier.as_str());
    let is_url_import = URL_REGEX.is_match(&specifier)
      || match base {
        Some(base) => URL_REGEX.is_match(base),
        None => false,
      };

    match (is_core_module_import, is_url_import) {
      (true, _) if !ignore_core_modules => Box::new(CoreModuleLoader),
      (_, true) => Box::<UrlModuleLoader>::default(),
      _ => Box::new(FsModuleLoader),
    }
  };

  // Resolve module.
  loader.resolve(base, &specifier)
}
