//! Js runtime hooks: promise, import and import.meta, etc.

use crate::js::JsRuntime;
use crate::js::binding::set_exception_code;
use crate::js::binding::throw_type_error;
use crate::js::module::EsModuleFuture;
use crate::js::module::ModuleGraph;
use crate::js::module::ModuleStatus;
use crate::js::module::resolve_import;
use crate::js::pending;
use crate::prelude::*;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

/// Called during `Module::instantiate_module`, see:
/// - Rusty V8 API:
///   - <https://docs.rs/v8/latest/v8/struct.Module.html#method.instantiate_module>.
///   - <https://docs.rs/rusty_v8/latest/rusty_v8/type.ResolveModuleCallback.html>
/// - Node V8 API: <https://v8docs.nodesource.com/node-24.1/df/d74/classv8_1_1_module.html#a3313f8faa14b6dc5d37c340d45273bf1>.
pub fn module_resolve_cb<'a>(
  context: v8::Local<'a, v8::Context>,
  specifier: v8::Local<'a, v8::String>,
  _import_attributes: v8::Local<'a, v8::FixedArray>,
  referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
  // Get `CallbackScope` from context.
  let scope = &mut unsafe { v8::CallbackScope::new(context) };
  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();

  let import_map = state.options.import_map.clone();
  let referrer = v8::Global::new(scope, referrer);

  let dependant = state.module_map.get_path(referrer);
  let dependant = dependant
    .map(|dep| {
      let dep = dep.clone();
      let p = Path::new(&dep).to_path_buf();
      p.parent().unwrap_or(p.as_path()).to_path_buf()
    })
    .map(|dep| dep.as_path().as_os_str().to_str().unwrap().to_string());

  let specifier = specifier.to_rust_string_lossy(scope);
  let specifier =
    resolve_import(dependant.as_deref(), &specifier, import_map).unwrap();
  trace!(
    "|module_resolve_cb| dependant:{:?}, specifier:{:?}",
    dependant, specifier
  );

  // This call should always give us back the module.
  let module = state.module_map.get(&specifier).unwrap();

  Some(v8::Local::new(scope, module))
}

/// Called the first time import.meta is accessed for a module.
/// See: <https://docs.rs/v8/0.49.0/v8/type.HostInitializeImportMetaObjectCallback.html>.
pub extern "C" fn host_initialize_import_meta_object_cb(
  context: v8::Local<v8::Context>,
  module: v8::Local<v8::Module>,
  meta: v8::Local<v8::Object>,
) {
  // Get `CallbackScope` from context.
  let scope = &mut unsafe { v8::CallbackScope::new(context) };
  let scope = &mut v8::HandleScope::new(scope);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();

  // Make the module global.
  let module = v8::Global::new(scope, module);

  let url = state.module_map.get_path(module).unwrap();
  let is_main = state.module_map.main().clone() == Some(url.to_owned());
  trace!(
    "|host_initialize_import_meta_object_cb| url:{:?}, is_main:{:?}",
    url, is_main
  );

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
  let value =
    v8::FunctionBuilder::<v8::Function>::build(builder, scope).unwrap();
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
  let base = Path::new(&base).parent().unwrap_or(Path::new(&base));
  let base = base.as_os_str().to_str().unwrap();
  let specifier = args.get(0).to_rust_string_lossy(scope);
  trace!(
    "|import_meta_resolve| base:{:?}, specifier:{:?}",
    base, specifier
  );
  let import_map = JsRuntime::state(scope).borrow().options.import_map.clone();

  match resolve_import(Some(base), &specifier, import_map) {
    Ok(path) => rv.set(v8::String::new(scope, &path).unwrap().into()),
    Err(e) => throw_type_error(scope, &e.to_string()),
  };
}

/// Called when a promise rejects with no rejection handler specified.
/// See: <https://docs.rs/v8/0.49.0/v8/type.PromiseRejectCallback.html>.
/// See: <https://v8.dev/features/promise-combinators>.
pub extern "C" fn promise_reject_cb(message: v8::PromiseRejectMessage) {
  // Create a v8 callback-scope.
  let scope = &mut unsafe { v8::CallbackScope::new(&message) };
  let undefined = v8::undefined(scope).into();
  let event = message.get_event();
  trace!("|promise_reject_cb| event:{event:?}, message:{message:?}");

  use v8::PromiseRejectEvent::PromiseHandlerAddedAfterReject;
  use v8::PromiseRejectEvent::PromiseRejectAfterResolved;
  use v8::PromiseRejectEvent::PromiseRejectWithNoHandler;
  use v8::PromiseRejectEvent::PromiseResolveAfterResolved;

  let reason = match event {
    PromiseHandlerAddedAfterReject
    | PromiseRejectAfterResolved
    | PromiseResolveAfterResolved => undefined,
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
      // state.interrupt_handle.interrupt();
    }
    PromiseHandlerAddedAfterReject => {
      state.exceptions.remove_promise_rejection(&promise);
    }
    PromiseRejectAfterResolved | PromiseResolveAfterResolved => {}
  }
}

// Called when we require the embedder to load a module.
// https://docs.rs/v8/0.56.1/v8/trait.HostImportModuleDynamicallyCallback.html
pub fn host_import_module_dynamically_cb<'s>(
  scope: &mut v8::HandleScope<'s>,
  _host_defined_options: v8::Local<'s, v8::Data>,
  base: v8::Local<'s, v8::Value>,
  specifier: v8::Local<'s, v8::String>,
  _import_attributes: v8::Local<v8::FixedArray>,
) -> Option<v8::Local<'s, v8::Promise>> {
  // Get module base and specifier as strings.
  let base = base.to_rust_string_lossy(scope);
  let base = Path::new(&base).parent().unwrap_or(Path::new(&base));
  let base = base.as_os_str().to_str().unwrap();
  let specifier = specifier.to_rust_string_lossy(scope);
  trace!(
    "|host_import_module_dynamically_cb| base:{:?}, specifier:{:?}",
    base, specifier
  );

  // Create the import promise.
  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let mut state = state_rc.borrow_mut();

  let import_map = state.options.import_map.clone();

  let specifier = match resolve_import(Some(&base), &specifier, import_map) {
    Ok(specifier) => specifier,
    Err(e) => {
      drop(state);
      trace!("Failed to resolve import {specifier:?}(base:{base:?}): {e:?}");
      let exception = v8::String::new(scope, &e.to_string()).unwrap();
      let exception = v8::Exception::error(scope, exception);
      set_exception_code(scope, exception, &e);
      promise_resolver.reject(scope, exception);
      return Some(promise);
    }
  };

  let dynamic_import_being_fetched =
    state.module_map.pending.iter().any(|graph_rc| {
      *graph_rc.borrow().root_rc().borrow().path() == specifier
    });

  // Check if the requested dynamic module is already resolved.
  if state.module_map.contains(&specifier) && !dynamic_import_being_fetched {
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
      .find(|graph_rc| {
        *graph_rc.borrow().root_rc().borrow().path() == specifier
      })
      .unwrap()
      .borrow_mut()
      .same_origin_mut()
      .push(global_promise);

    return Some(promise);
  }

  let graph = ModuleGraph::dynamic_import(&specifier, global_promise);
  let graph_rc = Rc::new(RefCell::new(graph));
  let status = ModuleStatus::Fetching;

  state.module_map.pending.push(Rc::clone(&graph_rc));
  trace!(
    "|host_import_module_dynamically_cb| ModuleMap pending {:?}",
    specifier
  );

  state.module_map.seen.insert(specifier.clone(), status);
  trace!(
    "|host_import_module_dynamically_cb| ModuleMap seen {:?} {:?}",
    specifier, status
  );

  // Use the event-loop to asynchronously load the requested module.
  let loader_cb = {
    let state_rc = state_rc.clone();
    let specifier = specifier.clone();
    move |maybe_result: Option<AnyResult<Vec<u8>>>| {
      let fut = EsModuleFuture {
        path: specifier.clone(),
        module: graph_rc.borrow().root_rc(),
        maybe_source: maybe_result,
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.insert(0, Box::new(fut));
    }
  };
  pending::create_import_loader(&mut state, &specifier, Box::new(loader_cb));

  Some(promise)
}
