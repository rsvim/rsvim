//! ECMAScript (ES) module, i.e. the module specified by keyword `import`.

use crate::js::err::JsError;
use crate::js::module::create_origin;
use crate::js::module::{ModulePath, ModuleStatus};
use crate::js::{JsFuture, JsRuntime};
use crate::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
/// ES Module.
pub struct EsModule {
  /// Module path on local file system.
  path: ModulePath,
  /// Module import status.
  status: ModuleStatus,
  /// Maps the module itself to all its dependencies.
  dependencies: Vec<EsModuleRc>,
  /// Exceptions when import.
  exception: Option<String>,
  /// Whether this module is dynamically import.
  is_dynamic_import: bool,
}

rc_refcell_ptr!(EsModule);

impl EsModule {
  pub fn new(
    path: ModulePath,
    status: ModuleStatus,
    dependencies: Vec<EsModuleRc>,
    exception: Option<String>,
    is_dynamic_import: bool,
  ) -> Self {
    Self {
      path,
      status,
      dependencies,
      exception,
      is_dynamic_import,
    }
  }

  pub fn path(&self) -> &ModulePath {
    &self.path
  }

  pub fn status(&self) -> ModuleStatus {
    self.status
  }

  pub fn set_status(&mut self, status: ModuleStatus) {
    self.status = status;
  }

  pub fn dependencies(&self) -> &Vec<EsModuleRc> {
    &self.dependencies
  }

  pub fn dependencies_mut(&mut self) -> &mut Vec<EsModuleRc> {
    &mut self.dependencies
  }

  pub fn exception(&self) -> &Option<String> {
    &self.exception
  }

  pub fn exception_mut(&mut self) -> &mut Option<String> {
    &mut self.exception
  }

  pub fn is_dynamic_import(&self) -> bool {
    self.is_dynamic_import
  }
}

impl EsModule {
  // Traverses the dependency tree to check if the module is ready.
  pub fn fast_forward(
    &mut self,
    seen_modules: &mut HashMap<ModulePath, ModuleStatus>,
  ) {
    // If the module is ready, no need to check the sub-tree.
    if self.status == ModuleStatus::Ready {
      return;
    }

    // If it's a duplicate module we need to check the module status cache.
    if self.status == ModuleStatus::Duplicate {
      let status_ref = seen_modules.get(&self.path).unwrap();
      if status_ref == &ModuleStatus::Ready {
        self.status = ModuleStatus::Ready;
      }
      return;
    }

    // Fast-forward all dependencies.
    self
      .dependencies
      .iter_mut()
      .for_each(|dep| dep.borrow_mut().fast_forward(seen_modules));

    // The module is compiled and has 0 dependencies.
    if self.dependencies.is_empty() && self.status == ModuleStatus::Resolving {
      self.status = ModuleStatus::Ready;
      seen_modules.insert(self.path.clone(), self.status);
      return;
    }

    // At this point, the module is still being fetched...
    if self.dependencies.is_empty() {
      return;
    }

    if !self
      .dependencies
      .iter_mut()
      .map(|m| m.borrow().status)
      .any(|status| status != ModuleStatus::Ready)
    {
      self.status = ModuleStatus::Ready;
      seen_modules.insert(self.path.clone(), self.status);
    }
  }
}

pub struct EsModuleFuture {
  pub path: ModulePath,
  pub module: Rc<RefCell<EsModule>>,
  pub maybe_result: Option<IoResult<String>>,
}

impl EsModuleFuture {
  // Handles static import error.
  fn handle_failure(&mut self, e: anyhow::Error) {
    let mut module = self.module.borrow_mut();
    // In dynamic imports we reject the promise(s).
    if module.is_dynamic_import() {
      module.exception_mut().replace(e.to_string());
      return;
    }

    // In static imports we exit the process.
    // FIXME: Only send the error to command-line, instead of exit process.
    eprintln!("{e}");
    std::process::exit(1);
  }
}

impl JsFuture for EsModuleFuture {
  /// Drives the future to completion.
  fn run(&mut self, scope: &mut v8::HandleScope) {
    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();

    // If the graph has exceptions, stop resolving the current sub-tree (dynamic imports).
    if self.module.borrow().exception().is_some() {
      state.module_map.seen().borrow_mut().remove(&self.path);
      return;
    }

    // Extract module's source code.
    let source = self.maybe_result.take().unwrap();
    let source = match source {
      Ok(source) => source,
      Err(e) => {
        self.handle_failure(anyhow::Error::msg(e.to_string()));
        return;
      }
    };

    let tc_scope = &mut v8::TryCatch::new(scope);
    let origin = create_origin(tc_scope, &self.path, true);

    // Compile source and get it's dependencies.
    let source = v8::String::new(tc_scope, &source).unwrap();
    let mut source = v8::script_compiler::Source::new(source, Some(&origin));

    let module =
      match v8::script_compiler::compile_module(tc_scope, &mut source) {
        Some(module) => module,
        None => {
          assert!(tc_scope.has_caught());
          let exception = tc_scope.exception().unwrap();
          let exception = JsError::from_v8_exception(tc_scope, exception, None);
          let exception =
            format!("{} ({})", exception.message, exception.resource_name);

          self.handle_failure(anyhow::Error::msg(exception));
          return;
        }
      };

    let new_status = ModuleStatus::Resolving;
    let module_ref = v8::Global::new(tc_scope, module);

    state.module_map.insert(self.path.as_str(), module_ref);
    state
      .module_map
      .seen()
      .borrow_mut()
      .insert(self.path.clone(), new_status);

    let import_map = state.options.import_map.clone();

    let skip_cache = match self.module.borrow().is_dynamic_import {
      true => !state.options.test_mode || state.options.reload,
      false => state.options.reload,
    };

    let mut dependencies = vec![];

    let requests = module.get_module_requests();
    let base = self.path.clone();

    for i in 0..requests.length() {
      // Get import request from the `module_requests` array.
      let request = requests.get(tc_scope, i).unwrap();
      let request = v8::Local::<v8::ModuleRequest>::try_from(request).unwrap();

      // Transform v8's ModuleRequest into Rust string.
      let base = Some(base.as_str());
      let specifier = request.get_specifier().to_rust_string_lossy(tc_scope);
      let specifier =
        match resolve_import(base, &specifier, false, import_map.clone()) {
          Ok(specifier) => specifier,
          Err(e) => {
            self.handle_failure(Error::msg(e.to_string()));
            return;
          }
        };

      // Check if requested module has been seen already.
      let seen_module = state.module_map.seen.get(&specifier);
      let status = match seen_module {
        Some(ModuleStatus::Ready) => continue,
        Some(_) => ModuleStatus::Duplicate,
        None => ModuleStatus::Fetching,
      };

      // Create a new ES module instance.
      let module = Rc::new(RefCell::new(EsModule {
        path: specifier.clone(),
        status,
        dependencies: vec![],
        exception: Rc::clone(&self.module.borrow().exception),
        is_dynamic_import: self.module.borrow().is_dynamic_import,
      }));

      dependencies.push(Rc::clone(&module));

      // If the module is newly seen, use the event-loop to load
      // the requested module.
      if seen_module.is_none() {
        let task = {
          let specifier = specifier.clone();
          move || match load_import(&specifier, skip_cache) {
            Ok(source) => Some(Ok(bincode::serialize(&source).unwrap())),
            Err(e) => Some(Result::Err(e)),
          }
        };

        let task_cb = {
          let specifier = specifier.clone();
          let state_rc = state_rc.clone();
          move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = EsModuleFuture {
              path: specifier,
              module: Rc::clone(&module),
              maybe_result,
            };
            state.pending_futures.push(Box::new(future));
          }
        };

        state.module_map.seen.insert(specifier, status);
        state.handle.spawn(task, Some(task_cb));
      }
    }

    self.module.borrow_mut().status = ModuleStatus::Resolving;
    self.module.borrow_mut().dependencies = dependencies;
  }
}
