//! JavaScript runtime.

use crate::buf::BuffersArc;
use crate::cli::CliOpt;
use crate::error::{AnyErr, TheErr};
use crate::js::err::JsError;
use crate::js::exception::ExceptionState;
use crate::js::hook::module_resolve_cb;
use crate::js::module::{
  create_origin, fetch_module_tree, resolve_import, ImportKind, ImportMap, ModuleMap, ModuleStatus,
};
use crate::js::msg::{EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage};
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::sync::Once;
use std::time::Instant;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error};

pub mod binding;
pub mod constant;
pub mod err;
pub mod exception;
pub mod hook;
pub mod loader;
pub mod module;
pub mod msg;
pub mod transpiler;

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct JsRuntimeOptions {
  // // The seed used in Math.random() method.
  // pub seed: Option<i64>,
  // // Reloads every URL import.
  // pub reload: bool,
  // The main entry point for the program.
  pub root: Option<String>,
  // Holds user defined import maps for module loading.
  pub import_map: Option<ImportMap>,
  // // The numbers of threads used by the thread-pool.
  // pub num_threads: Option<usize>,
  // Indicates if we're running JavaScript tests.
  pub test_mode: bool,
  // // Defines the inspector listening options.
  // pub inspect: Option<(SocketAddrV4, bool)>,
  // // Exposes v8's garbage collector.
  // pub expose_gc: bool,
  /// V8 flags.
  pub v8_flags: Vec<String>,
}

// /// A vector with JS callbacks and parameters.
// type NextTickQueue = Vec<(v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>)>;

/// An abstract interface for javascript `Promise` and `async`.
/// since everything in V8 needs the `&mut v8::HandleScope` to operate with, we cannot simply put
/// the async task into tokio `spawn` API, but to first
pub trait JsFuture {
  fn run(&mut self, scope: &mut v8::HandleScope);
}

pub type JsFutureId = i32;

/// Next future/task ID for js runtime.
///
/// NOTE: Start form 1.
pub fn next_future_id() -> JsFutureId {
  static GLOBAL: AtomicI32 = AtomicI32::new(1);
  GLOBAL.fetch_add(1, Ordering::Relaxed)
}

pub struct JsRuntimeStateForSnapshot {
  /// A sand-boxed execution context with its own set of built-in objects and functions.
  pub context: v8::Global<v8::Context>,
  /// Runtime options.
  pub options: JsRuntimeOptions,
}

/// Js runtime for creating V8 snapshot.
pub struct JsRuntimeForSnapshot {
  pub isolate: ManuallyDrop<v8::OwnedIsolate>,
}

impl JsRuntimeForSnapshot {
  /// Creates a new JsRuntime for V8 snapshot.
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    // Fire up the v8 engine.
    init_v8_platform();

    let mut isolate = v8::Isolate::snapshot_creator(None, None);

    let context = {
      let scope = &mut v8::HandleScope::new(&mut *isolate);
      let context = binding::create_new_context(scope);
      scope.set_default_context(context);
      v8::Global::new(scope, context)
    };

    // Load all runtime modules
    JsRuntimeForSnapshot::init_environment(&mut isolate, context);

    JsRuntimeForSnapshot {
      isolate: ManuallyDrop::new(isolate),
    }
  }

  /// Initializes synchronously the core environment (see js/runtime/global.js).
  fn init_environment(isolate: &mut v8::OwnedIsolate, context: v8::Global<v8::Context>) {
    let mut scope = v8::HandleScope::with_context(isolate, context);

    let name = "rsvim:runtime/10__web.js";
    let source = include_str!("./js/runtime/10__web.js");
    JsRuntimeForSnapshot::init_builtin_module(&mut scope, name, source);

    let name = "rsvim:runtime/50__rsvim.js";
    let source = include_str!("./js/runtime/50__rsvim.js");
    JsRuntimeForSnapshot::init_builtin_module(&mut scope, name, source);
  }

  /// Synchronously load builtin module.
  fn init_builtin_module(scope: &mut v8::HandleScope, name: &str, source: &str) {
    let tc_scope = &mut v8::TryCatch::new(scope);

    let module = match fetch_module_tree(tc_scope, name, Some(source)) {
      Some(module) => module,
      None => {
        assert!(tc_scope.has_caught());
        let exception = tc_scope.exception().unwrap();
        let exception = JsError::from_v8_exception(tc_scope, exception, None);
        error!("Failed to import builtin modules: {name}, error: {exception:?}");
        eprintln!("Failed to import builtin modules: {name}, error: {exception:?}");
        std::process::exit(1);
      }
    };

    if module
      .instantiate_module(tc_scope, module_resolve_cb)
      .is_none()
    {
      assert!(tc_scope.has_caught());
      let exception = tc_scope.exception().unwrap();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      error!("Failed to instantiate builtin modules: {name}, error: {exception:?}");
      eprintln!("Failed to instantiate builtin modules: {name}, error: {exception:?}");
      std::process::exit(1);
    }

    let _ = module.evaluate(tc_scope);

    if module.get_status() == v8::ModuleStatus::Errored {
      let exception = module.get_exception();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      error!("Failed to evaluate builtin modules: {name}, error: {exception:?}");
      eprintln!("Failed to evaluate builtin modules: {name}, error: {exception:?}");
      std::process::exit(1);
    }
  }
}

pub struct JsRuntimeState {
  /// A sand-boxed execution context with its own set of built-in objects and functions.
  pub context: v8::Global<v8::Context>,
  /// Holds information about resolved ES modules.
  pub module_map: ModuleMap,
  /// Timeout handles, i.e. timer IDs.
  pub timeout_handles: HashSet<i32>,
  // /// A handle to the event-loop that can interrupt the poll-phase.
  // pub interrupt_handle: LoopInterruptHandle,
  /// Holds JS pending futures scheduled by the event-loop.
  pub pending_futures: HashMap<JsFutureId, Box<dyn JsFuture>>,
  /// Indicates the start time of the process.
  pub startup_moment: Instant,
  /// Specifies the timestamp which the current process began in Unix time.
  pub time_origin: u128,
  // /// Holds callbacks scheduled by nextTick.
  // pub next_tick_queue: NextTickQueue,
  /// Stores and manages uncaught exceptions.
  pub exceptions: ExceptionState,
  /// Runtime options.
  pub options: JsRuntimeOptions,
  // /// Tracks wake event for current loop iteration.
  // pub wake_event_queued: bool,

  // Data Access for RSVIM {
  // Js runtime ==request==> master.
  pub js_runtime_send_to_master: Sender<JsRuntimeToEventLoopMessage>,
  // Js runtime <==response== master.
  pub js_runtime_recv_from_master: Receiver<EventLoopToJsRuntimeMessage>,
  pub cli_opt: CliOpt,
  pub runtime_path: Arc<RwLock<Vec<PathBuf>>>,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
  // Same as the `state` in EventLoop.
  pub editing_state: StateArc,
  // Data Access for RSVIM }
}

// Initialize V8 platform.
pub fn init_v8_platform() {
  static V8_INIT: Once = Once::new();
  V8_INIT.call_once(move || {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
  });
}

pub struct JsRuntime {
  // V8 isolate.
  isolate: v8::OwnedIsolate,

  /// The state of the runtime.
  #[allow(unused)]
  pub state: Rc<RefCell<JsRuntimeState>>,
}

impl JsRuntime {
  /// Creates a new JsRuntime based on provided options.
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    options: JsRuntimeOptions,
    startup_moment: Instant,
    time_origin: u128,
    js_runtime_send_to_master: Sender<JsRuntimeToEventLoopMessage>,
    js_runtime_recv_from_master: Receiver<EventLoopToJsRuntimeMessage>,
    cli_opt: CliOpt,
    runtime_path: Arc<RwLock<Vec<PathBuf>>>,
    tree: TreeArc,
    buffers: BuffersArc,
    editing_state: StateArc,
  ) -> Self {
    // Configuration flags for V8.
    // let mut flags = String::from(concat!(
    //   " --no-validate-asm",
    //   " --turbo_fast_api_calls",
    //   " --harmony-temporal",
    //   " --js-float16array",
    // ));
    let flags = options.v8_flags.join(" ");
    v8::V8::set_flags_from_string(&flags);

    // Fire up the v8 engine.
    init_v8_platform();

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());

    // NOTE: Set microtasks policy to explicit, this requires we invoke `perform_microtask_checkpoint` API on each tick.
    // See: [`run_next_tick_callbacks`].
    // isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
    isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);
    isolate.set_promise_reject_callback(hook::promise_reject_cb);
    // isolate.set_host_import_module_dynamically_callback(hook::host_import_module_dynamically_cb);
    isolate
      .set_host_initialize_import_meta_object_callback(hook::host_initialize_import_meta_object_cb);

    let context = {
      let scope = &mut v8::HandleScope::new(&mut *isolate);
      let context = binding::create_new_context(scope);
      v8::Global::new(scope, context)
    };

    // const MIN_POOL_SIZE: usize = 1;

    // let event_loop = match options.num_threads {
    //   Some(n) => EventLoop::new(cmp::max(n, MIN_POOL_SIZE)),
    //   None => EventLoop::default(),
    // };

    // Initialize the v8 inspector.
    // let address = options.inspect.map(|(address, _)| (address));
    // let inspector = options.inspect.map(|(_, waiting_for_session)| {
    //   JsRuntimeInspector::new(
    //     &mut isolate,
    //     context.clone(),
    //     event_loop.interrupt_handle(),
    //     waiting_for_session,
    //     options.root.clone(),
    //   )
    // });

    // Store state inside the v8 isolate slot.
    // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
    let state = Rc::new(RefCell::new(JsRuntimeState {
      context,
      module_map: ModuleMap::new(),
      timeout_handles: HashSet::new(),
      // interrupt_handle: event_loop.interrupt_handle(),
      pending_futures: HashMap::new(),
      // timeout_queue: BTreeMap::new(),
      startup_moment,
      time_origin,
      // next_tick_queue: Vec::new(),
      exceptions: exception::ExceptionState::new(),
      options,
      // wake_event_queued: false,
      js_runtime_send_to_master,
      js_runtime_recv_from_master,
      cli_opt,
      runtime_path,
      tree,
      buffers,
      editing_state,
    }));

    isolate.set_slot(state.clone());

    JsRuntime {
      isolate,
      // event_loop,
      state,
      // inspector,
    }

    // runtime.load_main_environment();
    //
    // // Start inspector agent is requested.
    // if let Some(inspector) = runtime.inspector().as_mut() {
    //   let address = address.unwrap();
    //   inspector.borrow_mut().start_agent(address);
    // }
    //
    // runtime
  }

  /// Initializes synchronously the core environment (see js/runtime/global.js).
  pub fn init_environment(&mut self) {
    let name = "rsvim:runtime/10__web.js";
    let source = include_str!("./js/runtime/10__web.js");
    self.init_builtin_module(name, source);

    let name = "rsvim:runtime/50__rsvim.js";
    let source = include_str!("./js/runtime/50__rsvim.js");
    self.init_builtin_module(name, source);

    // // Initialize process static values.
    // process::refresh(tc_scope);
  }

  /// Synchronously load builtin module.
  fn init_builtin_module(&mut self, name: &str, source: &str) {
    let scope = &mut self.handle_scope();
    let tc_scope = &mut v8::TryCatch::new(scope);

    let module = match fetch_module_tree(tc_scope, name, Some(source)) {
      Some(module) => module,
      None => {
        assert!(tc_scope.has_caught());
        let exception = tc_scope.exception().unwrap();
        let exception = JsError::from_v8_exception(tc_scope, exception, None);
        error!("Failed to import builtin modules: {name}, error: {exception:?}");
        eprintln!("Failed to import builtin modules: {name}, error: {exception:?}");
        std::process::exit(1);
      }
    };

    if module
      .instantiate_module(tc_scope, module_resolve_cb)
      .is_none()
    {
      assert!(tc_scope.has_caught());
      let exception = tc_scope.exception().unwrap();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      error!("Failed to instantiate builtin modules: {name}, error: {exception:?}");
      eprintln!("Failed to instantiate builtin modules: {name}, error: {exception:?}");
      std::process::exit(1);
    }

    let _ = module.evaluate(tc_scope);

    if module.get_status() == v8::ModuleStatus::Errored {
      let exception = module.get_exception();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      error!("Failed to evaluate builtin modules: {name}, error: {exception:?}");
      eprintln!("Failed to evaluate builtin modules: {name}, error: {exception:?}");
      std::process::exit(1);
    }

    // // Initialize process static values.
    // process::refresh(tc_scope);
  }

  /// Executes traditional JavaScript code (traditional = not ES modules).
  ///
  /// NOTE: We don't use it.
  pub fn __execute_script(
    &mut self,
    filename: &str,
    source: &str,
  ) -> Result<Option<v8::Global<v8::Value>>, AnyErr> {
    // Get the handle-scope.
    let scope = &mut self.handle_scope();
    let state_rc = JsRuntime::state(scope);

    let origin = create_origin(scope, filename, false);
    let source = v8::String::new(scope, source).unwrap();

    // The `TryCatch` scope allows us to catch runtime errors rather than panicking.
    let tc_scope = &mut v8::TryCatch::new(scope);
    type ExecuteScriptResult = Result<Option<v8::Global<v8::Value>>, AnyErr>;

    let handle_exception =
      |scope: &mut v8::TryCatch<'_, v8::HandleScope<'_>>| -> ExecuteScriptResult {
        // Extract the exception during compilation.
        assert!(scope.has_caught());
        let exception = scope.exception().unwrap();
        let exception = v8::Global::new(scope, exception);
        let mut state = state_rc.borrow_mut();
        // Capture the exception internally.
        state.exceptions.capture_exception(exception);
        drop(state);
        // Force an exception check.
        if let Some(error) = check_exceptions(scope) {
          anyhow::bail!(error)
        }
        Ok(None)
      };

    let script = match v8::Script::compile(tc_scope, source, Some(&origin)) {
      Some(script) => script,
      None => return handle_exception(tc_scope),
    };

    match script.run(tc_scope) {
      Some(value) => Ok(Some(v8::Global::new(tc_scope, value))),
      None => handle_exception(tc_scope),
    }
  }

  /// Executes JavaScript code as ES module.
  pub fn execute_module(&mut self, filename: &str, source: Option<&str>) -> Result<(), AnyErr> {
    // Get a reference to v8's scope.
    let scope = &mut self.handle_scope();

    // The following code allows the runtime to execute code with no valid
    // location passed as parameter as an ES module.
    let path = match source.is_some() {
      true => filename.to_string(),
      false => match resolve_import(None, filename, false, None) {
        Ok(specifier) => specifier,
        Err(e) => {
          // Returns the error directly.
          return Err(e);
        }
      },
    };
    debug!("Resolved main js module (path): {:?}", path);

    let tc_scope = &mut v8::TryCatch::new(scope);

    // NOTE: Here we also use static module fetching, i.e. all the modules are already stored on
    // local file system, no network/http downloading will be involved.
    let module = match fetch_module_tree(tc_scope, filename, None) {
      Some(module) => module,
      None => {
        assert!(tc_scope.has_caught());
        let exception = tc_scope.exception().unwrap();
        let _exception = JsError::from_v8_exception(tc_scope, exception, None);
        let err_msg = format!("User config not found: {filename:?}");
        error!(err_msg);
        eprintln!("{err_msg}");
        return Err(TheErr::Message(err_msg).into());
      }
    };

    if module
      .instantiate_module(tc_scope, module_resolve_cb)
      .is_none()
    {
      assert!(tc_scope.has_caught());
      let exception = tc_scope.exception().unwrap();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      let err_msg = format!("Failed to instantiate user config module {filename:?}: {exception:?}");
      error!(err_msg);
      eprintln!("{err_msg}");
      return Err(TheErr::Message(err_msg).into());
    }

    match module.evaluate(tc_scope) {
      Some(result) => {
        debug!(
          "Evaluated user config module result ({:?}): {:?}",
          result.type_repr(),
          result.to_rust_string_lossy(tc_scope),
        );
      }
      None => debug!("Evaluated user config module result: None"),
    }

    if module.get_status() == v8::ModuleStatus::Errored {
      let exception = module.get_exception();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      let err_msg = format!("Failed to evaluate user config module {filename:?}: {exception:?}");
      error!(err_msg);
      eprintln!("{err_msg}");
      return Err(TheErr::Message(err_msg).into());
    }

    Ok(())
  }

  /// Runs a single tick of the event-loop.
  pub fn tick_event_loop(&mut self) {
    let isolate_has_pending_tasks = self.isolate.has_pending_background_tasks();
    debug!(
      "Tick js runtime, isolate has pending tasks: {:?}",
      isolate_has_pending_tasks
    );
    run_next_tick_callbacks(&mut self.handle_scope());
    self.fast_forward_imports();
    // self.event_loop.tick();
    self.run_pending_futures();
    debug!("Tick js runtime - done");
  }

  // /// Polls the inspector for new devtools messages.
  // pub fn poll_inspect_session(&mut self) {
  //   if let Some(inspector) = self.inspector.as_mut() {
  //     inspector.borrow_mut().poll_session();
  //   }
  // }

  // /// Runs the event-loop until no more pending events exists.
  // pub fn run_event_loop(&mut self) {
  //   // Check for pending devtools messages.
  //   self.poll_inspect_session();
  //   // Run callbacks/promises from next-tick and micro-task queues.
  //   run_next_tick_callbacks(&mut self.handle_scope());
  //
  //   while self.event_loop.has_pending_events()
  //     || self.has_promise_rejections()
  //     || self.isolate.has_pending_background_tasks()
  //     || self.has_pending_imports()
  //     || self.has_next_tick_callbacks()
  //   {
  //     // Check for pending devtools messages.
  //     self.poll_inspect_session();
  //     // Tick the event-loop one cycle.
  //     self.tick_event_loop();
  //
  //     // Report any unhandled promise rejections.
  //     if let Some(error) = check_exceptions(&mut self.handle_scope()) {
  //       report_and_exit(error);
  //     }
  //   }
  //
  //   // We can now notify debugger that the program has finished running
  //   // and we're ready to exit the process.
  //   if let Some(inspector) = self.inspector() {
  //     let context = self.context();
  //     let scope = &mut self.handle_scope();
  //     inspector.borrow_mut().context_destroyed(scope, context);
  //   }
  // }

  /// Runs pending javascript tasks which have received results from master.
  fn run_pending_futures(&mut self) {
    // Get a handle-scope and a reference to the runtime's state.
    let scope = &mut self.handle_scope();
    let mut futures: Vec<Box<dyn JsFuture>> = Vec::new();

    {
      let state_rc = Self::state(scope);
      let mut state = state_rc.borrow_mut();
      while let Ok(msg) = state.js_runtime_recv_from_master.try_recv() {
        match msg {
          EventLoopToJsRuntimeMessage::TimeoutResp(resp) => {
            match state.pending_futures.remove(&resp.future_id) {
              Some(timeout_cb) => futures.push(timeout_cb),
              None => unreachable!("Failed to get timeout future by ID {:?}", resp.future_id),
            }
          }
        }
      }

      // Drop borrowed `state_rc` or it will panics when running these futures.
    }

    for mut fut in futures {
      fut.run(scope);
      if let Some(error) = check_exceptions(scope) {
        // FIXME: Cannot simply report error and exit process, because this is inside the editor.
        error!("Js runtime timeout error:{error:?}");
        eprintln!("Js runtime timeout error:{error:?}");
      }
      run_next_tick_callbacks(scope);
    }
  }

  /// Checks for imports (static/dynamic) ready for execution.
  fn fast_forward_imports(&mut self) {
    // Get a v8 handle-scope.
    let scope = &mut self.handle_scope();
    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();

    let mut ready_imports = vec![];

    // Note: The following is a trick to get multiple `mut` references in the same
    // struct called splitting borrows (https://doc.rust-lang.org/nomicon/borrow-splitting.html).
    let state_ref = &mut *state;
    let pending_graphs = &mut state_ref.module_map.pending;
    let seen_modules = &mut state_ref.module_map.seen;

    pending_graphs.retain(|graph_rc| {
      // Get a usable ref to graph's root module.
      let graph = graph_rc.borrow();
      let mut graph_root = graph.root_rc.borrow_mut();

      // Check for exceptions in the graph (dynamic imports).
      if let Some(message) = graph_root.exception.borrow_mut().take() {
        // Create a v8 exception.
        let exception = v8::String::new(scope, &message).unwrap();
        let exception = v8::Exception::error(scope, exception);

        // We need to resolve all identical dynamic imports.
        match graph.kind.clone() {
          ImportKind::Static => unreachable!(),
          ImportKind::Dynamic(main_promise) => {
            for promise in [main_promise].iter().chain(graph.same_origin.iter()) {
              promise.open(scope).reject(scope, exception);
            }
          }
        }

        return false;
      }

      // If the graph is still loading, fast-forward the dependencies.
      if graph_root.status != ModuleStatus::Ready {
        graph_root.fast_forward(seen_modules);
        return true;
      }

      ready_imports.push(Rc::clone(graph_rc));
      false
    });

    // Note: We have to drop the sate ref here to avoid borrow panics
    // during the module instantiation/evaluation process.
    drop(state);

    // Execute the root module from the graph.
    for graph_rc in ready_imports {
      // Create a tc scope.
      let tc_scope = &mut v8::TryCatch::new(scope);

      let graph = graph_rc.borrow();
      let path = graph.root_rc.borrow().path.clone();

      let module = state_rc.borrow().module_map.get(&path).unwrap();
      let module = v8::Local::new(tc_scope, module);

      if module
        .instantiate_module(tc_scope, module_resolve_cb)
        .is_none()
      {
        assert!(tc_scope.has_caught());
        let exception = tc_scope.exception().unwrap();
        let exception = JsError::from_v8_exception(tc_scope, exception, None);
        // FIXME: Cannot simply report error and exit process, because this is inside the editor.
        error!("{exception:?}");
        eprintln!("{exception:?}");
        continue;
      }

      let _ = module.evaluate(tc_scope);
      let is_root_module = !graph.root_rc.borrow().is_dynamic_import;

      // Note: Due to the architecture, when a module errors, the `promise_reject_cb`
      // v8 hook will also trigger, resulting in the same exception being registered
      // as an unhandled promise rejection. Therefore, we need to manually remove it.
      if module.get_status() == v8::ModuleStatus::Errored && is_root_module {
        let mut state = state_rc.borrow_mut();
        let exception = module.get_exception();
        let exception = v8::Global::new(tc_scope, exception);

        state.exceptions.capture_exception(exception.clone());
        state.exceptions.remove_promise_rejection_entry(&exception);

        drop(state);

        if let Some(error) = check_exceptions(tc_scope) {
          // FIXME: Cannot simply report error and exit process, because this is inside the editor.
          error!("{error:?}");
          eprintln!("{error:?}");
          continue;
        }
      }

      if let ImportKind::Dynamic(main_promise) = graph.kind.clone() {
        // Note: Since this is a dynamic import will resolve the promise
        // with the module's namespace object instead of it's evaluation result.
        let namespace = module.get_module_namespace();

        // We need to resolve all identical dynamic imports.
        for promise in [main_promise].iter().chain(graph.same_origin.iter()) {
          promise.open(tc_scope).resolve(tc_scope, namespace);
        }
      }
    }

    // Note: It's important to perform a nextTick checkpoint at this
    // point to allow resources behind a promise to be scheduled correctly
    // to the event-loop.
    run_next_tick_callbacks(scope);
  }

  /// Returns if unhandled promise rejections where caught.
  pub fn has_promise_rejections(&mut self) -> bool {
    self.get_state().borrow().exceptions.has_promise_rejection()
  }

  /// Returns if we have imports in pending state.
  pub fn has_pending_imports(&mut self) -> bool {
    self.get_state().borrow().module_map.has_pending_imports()
  }

  // /// Returns if we have scheduled any next-tick callbacks.
  // pub fn has_next_tick_callbacks(&mut self) -> bool {
  //   !self.get_state().borrow().next_tick_queue.is_empty()
  // }
}

// State management specific methods.
// https://github.com/lmt-swallow/puppy-browser/blob/main/src/javascript/runtime.rs
impl JsRuntime {
  /// Returns the runtime state stored in the given isolate.
  pub fn state(isolate: &v8::Isolate) -> Rc<RefCell<JsRuntimeState>> {
    isolate
      .get_slot::<Rc<RefCell<JsRuntimeState>>>()
      .unwrap()
      .clone()
  }

  /// Returns the runtime's state.
  pub fn get_state(&self) -> Rc<RefCell<JsRuntimeState>> {
    Self::state(&self.isolate)
  }

  /// Returns a v8 handle scope for the runtime.
  /// See: <https://v8docs.nodesource.com/node-0.8/d3/d95/classv8_1_1_handle_scope.html>.
  pub fn handle_scope(&mut self) -> v8::HandleScope {
    let context = self.context();
    v8::HandleScope::with_context(&mut self.isolate, context)
  }

  /// Returns a context created for the runtime.
  /// See: <https://v8docs.nodesource.com/node-0.8/df/d69/classv8_1_1_context.html>.
  pub fn context(&mut self) -> v8::Global<v8::Context> {
    let state = self.get_state();
    let state = state.borrow();
    state.context.clone()
  }

  // /// Returns the inspector created for the runtime.
  // pub fn inspector(&mut self) -> Option<Rc<RefCell<JsRuntimeInspector>>> {
  //   self.inspector.as_ref().cloned()
  // }
}

/// Runs callbacks stored in the next-tick queue.
fn run_next_tick_callbacks(scope: &mut v8::HandleScope) {
  // let state_rc = JsRuntime::state(scope);
  // let callbacks: NextTickQueue = state_rc.borrow_mut().next_tick_queue.drain(..).collect();

  // let undefined = v8::undefined(scope);
  let tc_scope = &mut v8::TryCatch::new(scope);
  //
  // for (cb, params) in callbacks {
  //   // Create a local handle for the callback and its parameters.
  //   let cb = v8::Local::new(tc_scope, cb);
  //   let args: Vec<v8::Local<v8::Value>> = params
  //     .iter()
  //     .map(|arg| v8::Local::new(tc_scope, arg))
  //     .collect();
  //
  //   cb.call(tc_scope, undefined.into(), &args);
  //
  //   // On exception, report it and handle the error.
  //   if tc_scope.has_caught() {
  //     let exception = tc_scope.exception().unwrap();
  //     let exception = v8::Global::new(tc_scope, exception);
  //     let mut state = state_rc.borrow_mut();
  //     state.exceptions.capture_exception(exception);
  //
  //     drop(state);
  //
  //     // Check for uncaught errors (capture callbacks might be in place).
  //     if let Some(error) = check_exceptions(tc_scope) {
  //       report_and_exit(error);
  //     }
  //   }
  // }

  tc_scope.perform_microtask_checkpoint();
}

// Returns an error if an uncaught exception or unhandled rejection has been captured.
pub fn check_exceptions(scope: &mut v8::HandleScope) -> Option<JsError> {
  let state_rc = JsRuntime::state(scope);
  let maybe_exception = state_rc.borrow_mut().exceptions.exception.take();

  // Check for uncaught exceptions first.
  if let Some(exception) = maybe_exception {
    let state = state_rc.borrow();
    let exception = v8::Local::new(scope, exception);
    if let Some(callback) = state.exceptions.uncaught_exception_cb.as_ref() {
      let callback = v8::Local::new(scope, callback);
      let undefined = v8::undefined(scope).into();
      let origin = v8::String::new(scope, "uncaughtException").unwrap();
      let tc_scope = &mut v8::TryCatch::new(scope);
      drop(state);

      callback.call(tc_scope, undefined, &[exception, origin.into()]);

      // Note: To avoid infinite recursion with these hooks, if this
      // function throws, return it as error.
      if tc_scope.has_caught() {
        let exception = tc_scope.exception().unwrap();
        let exception = v8::Local::new(tc_scope, exception);
        let error = JsError::from_v8_exception(tc_scope, exception, None);
        return Some(error);
      }

      return None;
    }

    let error = JsError::from_v8_exception(scope, exception, None);
    return Some(error);
  }

  // let promise_rejections: Vec<PromiseRejectionEntry> = state_rc
  //   .borrow_mut()
  //   .exceptions
  //   .promise_rejections
  //   .drain(..)
  //   .collect();
  //
  // // Then, check for unhandled rejections.
  // for (promise, exception) in promise_rejections.iter() {
  //   let state = state_rc.borrow_mut();
  //   let promise = v8::Local::new(scope, promise);
  //   let exception = v8::Local::new(scope, exception);
  //
  //   // If the `unhandled_rejection_cb` is set, invoke it to handle the promise rejection.
  //   if let Some(callback) = state.exceptions.unhandled_rejection_cb.as_ref() {
  //     let callback = v8::Local::new(scope, callback);
  //     let undefined = v8::undefined(scope).into();
  //     let tc_scope = &mut v8::TryCatch::new(scope);
  //     drop(state);
  //
  //     callback.call(tc_scope, undefined, &[exception, promise.into()]);
  //
  //     // Note: To avoid infinite recursion with these hooks, if this
  //     // function throws, return it as error.
  //     if tc_scope.has_caught() {
  //       let exception = tc_scope.exception().unwrap();
  //       let exception = v8::Local::new(tc_scope, exception);
  //       let error = JsError::from_v8_exception(tc_scope, exception, None);
  //       return Some(error);
  //     }
  //
  //     continue;
  //   }
  //
  //   // If the `uncaught_exception_cb` is set, invoke it to handle the promise rejection.
  //   if let Some(callback) = state.exceptions.uncaught_exception_cb.as_ref() {
  //     let callback = v8::Local::new(scope, callback);
  //     let undefined = v8::undefined(scope).into();
  //     let origin = v8::String::new(scope, "unhandledRejection").unwrap();
  //     let tc_scope = &mut v8::TryCatch::new(scope);
  //     drop(state);
  //
  //     callback.call(tc_scope, undefined, &[exception, origin.into()]);
  //
  //     // Note: To avoid infinite recursion with these hooks, if this
  //     // function throws, return it as error.
  //     if tc_scope.has_caught() {
  //       let exception = tc_scope.exception().unwrap();
  //       let exception = v8::Local::new(tc_scope, exception);
  //       let error = JsError::from_v8_exception(tc_scope, exception, None);
  //       return Some(error);
  //     }
  //
  //     continue;
  //   }
  //
  //   let prefix = Some("(in promise) ");
  //   let error = JsError::from_v8_exception(scope, exception, prefix);
  //
  //   return Some(error);
  // }

  None
}

// /// Report unhandled exceptions and clear it.
// pub fn report_and_exit(e: JsError) {
//   error!("{:?}", e);
//   eprintln!("{:?}", e);
//   std::process::exit(1);
// }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn next_future_id1() {
    assert!(next_future_id() > 0);
  }
}
