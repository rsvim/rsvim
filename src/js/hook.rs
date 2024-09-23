//! Js runtime hooks: promise, import and import.meta, etc.

use crate::js::binding::{set_exception_code, throw_type_error};
// use crate::errors::unwrap_or_exit;
// use crate::modules::load_import;
use crate::js::module::resolve_import;
// use crate::modules::EsModuleFuture;
// use crate::modules::ModuleGraph;
// use crate::modules::ModuleStatus;
use crate::js::JsRuntime;
// use dune_event_loop::LoopHandle;
// use dune_event_loop::TaskResult;
use std::cell::RefCell;
use std::rc::Rc;

/// Called during Module::instantiate_module.
/// https://docs.rs/rusty_v8/latest/rusty_v8/type.ResolveModuleCallback.html
pub fn module_resolve_cb<'a>(
  context: v8::Local<'a, v8::Context>,
  specifier: v8::Local<'a, v8::String>,
  _: v8::Local<'a, v8::FixedArray>,
  referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
  // Get `CallbackScope` from context.
  let scope = &mut unsafe { v8::CallbackScope::new(context) };
  let state = JsRuntime::state(scope);
  let state = state.borrow();

  let import_map = state.options.import_map.clone();
  let referrer = v8::Global::new(scope, referrer);

  let dependant = state.module_map.get_path(referrer);

  let specifier = specifier.to_rust_string_lossy(scope);
  let specifier = resolve_import(dependant.as_deref(), &specifier, false, import_map).unwrap();

  // This call should always give us back the module.
  let module = state.module_map.get(&specifier).unwrap();

  Some(v8::Local::new(scope, module))
}

/// Called the first time import.meta is accessed for a module.
/// https://docs.rs/v8/0.49.0/v8/type.HostInitializeImportMetaObjectCallback.html
pub extern "C" fn host_initialize_import_meta_object_cb(
  context: v8::Local<v8::Context>,
  module: v8::Local<v8::Module>,
  meta: v8::Local<v8::Object>,
) {
  // Get `CallbackScope` from context.
  let scope = &mut unsafe { v8::CallbackScope::new(context) };
  let scope = &mut v8::HandleScope::new(scope);

  let state = JsRuntime::state(scope);
  let state = state.borrow();

  // Make the module global.
  let module = v8::Global::new(scope, module);

  let url = state.module_map.get_path(module).unwrap();
  let is_main = state.module_map.main() == Some(url.to_owned());

  // Setup import.url property.
  let key = v8::String::new(scope, "url").unwrap();
  let value = v8::String::new(scope, &url).unwrap();
  meta.create_data_property(scope, key.into(), value.into());

  // Setup import.main property.
  let key = v8::String::new(scope, "main").unwrap();
  let value = v8::Boolean::new(scope, is_main);
  meta.create_data_property(scope, key.into(), value.into());

  let url = v8::String::new(scope, &url).unwrap();
  let builder = v8::FunctionBuilder::new(import_meta_resolve).data(url.into());

  // Setup import.resolve() method.
  let key = v8::String::new(scope, "resolve").unwrap();
  let value = v8::FunctionBuilder::<v8::Function>::build(builder, scope).unwrap();
  meta.set(scope, key.into(), value.into());
}

fn import_meta_resolve(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  // Check for provided arguments.
  if args.length() == 0 {
    throw_type_error(scope, "Not enough arguments specified.");
    return;
  }

  let base = args.data().to_rust_string_lossy(scope);
  let specifier = args.get(0).to_rust_string_lossy(scope);
  let import_map = JsRuntime::state(scope).borrow().options.import_map.clone();

  match resolve_import(Some(&base), &specifier, false, import_map) {
    Ok(path) => rv.set(v8::String::new(scope, &path).unwrap().into()),
    Err(e) => throw_type_error(scope, &e.to_string()),
  };
}

/// Called when a promise rejects with no rejection handler specified.
/// https://docs.rs/v8/0.49.0/v8/type.PromiseRejectCallback.html
/// https://v8.dev/features/promise-combinators
pub extern "C" fn promise_reject_cb(message: v8::PromiseRejectMessage) {
  // Create a v8 callback-scope.
  let scope = &mut unsafe { v8::CallbackScope::new(&message) };
  let undefined = v8::undefined(scope).into();
  let event = message.get_event();

  use v8::PromiseRejectEvent::{
    PromiseHandlerAddedAfterReject, PromiseRejectAfterResolved, PromiseRejectWithNoHandler,
    PromiseResolveAfterResolved,
  };

  let reason = match event {
    PromiseHandlerAddedAfterReject | PromiseRejectAfterResolved | PromiseResolveAfterResolved => {
      undefined
    }
    PromiseRejectWithNoHandler => message.get_value().unwrap(),
  };

  let promise = message.get_promise();
  let promise = v8::Global::new(scope, promise);

  let state_rc = JsRuntime::state(scope);
  let mut state = state_rc.borrow_mut();

  match event {
    // Note: We might need to "interrupt" the event loop to handle
    // the promise rejection in a timely manner.
    PromiseRejectWithNoHandler => {
      let reason = v8::Global::new(scope, reason);
      state.exceptions.capture_promise_rejection(promise, reason);
      state.interrupt_handle.interrupt();
    }
    PromiseHandlerAddedAfterReject => {
      state.exceptions.remove_promise_rejection(&promise);
    }
    PromiseRejectAfterResolved | PromiseResolveAfterResolved => {}
  }
}

// Called when we require the embedder to load a module.
// https://docs.rs/v8/0.56.1/v8/trait.HostImportModuleDynamicallyCallback.html
// https://v8.dev/features/dynamic-import
pub fn host_import_module_dynamically_cb<'s>(
  scope: &mut v8::HandleScope<'s>,
  _: v8::Local<'s, v8::Data>,
  base: v8::Local<'s, v8::Value>,
  specifier: v8::Local<'s, v8::String>,
  _: v8::Local<v8::FixedArray>,
) -> Option<v8::Local<'s, v8::Promise>> {
  // Get module base and specifier as strings.
  let base = base.to_rust_string_lossy(scope);
  let specifier = specifier.to_rust_string_lossy(scope);

  // Create the import promise.
  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let mut state = state_rc.borrow_mut();

  let import_map = state.options.import_map.clone();

  let specifier = match resolve_import(Some(&base), &specifier, false, import_map) {
    Ok(specifier) => specifier,
    Err(e) => {
      drop(state);
      let exception = v8::String::new(scope, &e.to_string()[18..]).unwrap();
      let exception = v8::Exception::error(scope, exception);
      set_exception_code(scope, exception, &e);
      promise_resolver.reject(scope, exception);
      return Some(promise);
    }
  };

  let dynamic_import_being_fetched = state
    .module_map
    .pending
    .iter()
    .any(|graph_rc| graph_rc.borrow().root_rc.borrow().path == specifier);

  // Check if the requested dynamic module is already resolved.
  if state.module_map.index.contains_key(&specifier) && !dynamic_import_being_fetched {
    // Create a local handle for the module.
    let module = state.module_map.get(&specifier).unwrap();
    let module = module.open(scope);

    // Note: Since this is a dynamic import will resolve the promise
    // with the module's namespace object instead of it's evaluation result.
    promise_resolver.resolve(scope, module.get_module_namespace());
    return Some(promise);
  }

  let global_promise = v8::Global::new(scope, promise_resolver);

  if dynamic_import_being_fetched {
    // Find the graph with the same root that is being resolved
    // and declare this graph as same origin.
    state
      .module_map
      .pending
      .iter()
      .find(|graph_rc| graph_rc.borrow().root_rc.borrow().path == specifier)
      .unwrap()
      .borrow_mut()
      .same_origin
      .push_back(global_promise);

    return Some(promise);
  }

  let graph = ModuleGraph::dynamic_import(&specifier, global_promise);
  let graph_rc = Rc::new(RefCell::new(graph));
  let status = ModuleStatus::Fetching;

  state.module_map.pending.push(Rc::clone(&graph_rc));
  state.module_map.seen.insert(specifier.clone(), status);

  /*  Use the event-loop to asynchronously load the requested module. */

  let task = {
    let specifier = specifier.clone();
    move || match load_import(&specifier, true) {
      anyhow::Result::Ok(source) => Some(Ok(bincode::serialize(&source).unwrap())),
      Err(e) => Some(Result::Err(e)),
    }
  };

  let task_cb = {
    let state_rc = state_rc.clone();
    move |_: LoopHandle, maybe_result: TaskResult| {
      let mut state = state_rc.borrow_mut();
      let future = EsModuleFuture {
        path: specifier,
        module: Rc::clone(&graph_rc.borrow().root_rc),
        maybe_result,
      };
      state.pending_futures.push(Box::new(future));
    }
  };

  state.handle.spawn(task, Some(task_cb));

  Some(promise)
}
