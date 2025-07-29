//! JavaScript runtime.

use crate::buf::BuffersManagerArc;
use crate::cli::CliOpt;
use crate::content::TextContentsArc;
use crate::js::err::JsError;
use crate::js::exception::ExceptionState;
use crate::js::hook::module_resolve_cb;
use crate::js::module::{
  ImportKind, ImportMap, ModuleMap, ModuleStatus, fetch_module,
  fetch_module_tree, resolve_import,
};
use crate::js::msg::{
  EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage,
};
use crate::prelude::*;
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

use std::rc::Rc;
use std::sync::Once;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{error, trace};

pub mod binding;
pub mod err;
pub mod exception;
pub mod hook;
pub mod loader;
pub mod module;
pub mod msg;
pub mod transpiler;

#[cfg(test)]
mod module_tests;

pub fn v8_version() -> &'static str {
  v8::VERSION_STRING
}

// /// A vector with JS callbacks and parameters.
// type NextTickQueue = Vec<(v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>)>;

/// An abstract interface for javascript `Promise` and `async`.
/// Since everything in V8 needs the `&mut v8::HandleScope` to operate with, we cannot simply put
/// the async task into tokio `spawn` API.
pub trait JsFuture {
  fn run(&mut self, scope: &mut v8::HandleScope);
}

pub type JsFutureId = i32;

/// Next future/task ID for js runtime.
///
/// NOTE: Start form 1.
pub fn next_future_id() -> JsFutureId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

pub fn init_v8_platform(snapshot: bool, user_v8_flags: Option<&[String]>) {
  static V8_INIT: Once = Once::new();

  V8_INIT.call_once(move || {
    // Configuration flags for V8.
    // See: <https://github.com/denoland/deno_core/blob/3289dad2501818c838a76c203f73d0dd62ec6167/core/runtime/setup.rs#L72>.
    let mut flags = String::from(concat!(
      " --no-validate-asm",
      " --turbo-fast-api-calls",
      " --harmony-temporal",
      " --js-float16array",
      " --js-explicit-resource-management",
    ));

    if snapshot {
      flags.push_str(" --predictable --random-seed=42");
    }

    if let Some(user_flags) = user_v8_flags {
      if !user_flags.is_empty() {
        let user_flags = user_flags.join(" ");
        let user_flags = format!(" {user_flags}");
        flags.push_str(user_flags.as_str());
      }
    }

    v8::V8::set_flags_from_string(&flags);

    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
  });
}

fn init_v8_isolate(isolate: &mut v8::OwnedIsolate) {
  // NOTE: Set microtasks policy to explicit, this requires we invoke `perform_microtask_checkpoint` API on each tick.
  // See: [`run_next_tick_callbacks`].
  isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
  isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);
  isolate.set_promise_reject_callback(hook::promise_reject_cb);
  // isolate.set_host_import_module_dynamically_callback(hook::host_import_module_dynamically_cb);
  isolate.set_host_initialize_import_meta_object_callback(
    hook::host_initialize_import_meta_object_cb,
  );
}

fn init_builtin_modules(scope: &mut v8::HandleScope<'_>) {
  static BUILTIN_MODULES: [(/* filename */ &str, /* source */ &str); 2] = [
    (
      "00__web.js",
      include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/js/runtime/00__web.js"
      )),
    ),
    (
      "01__rsvim.js",
      include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/js/runtime/01__rsvim.js"
      )),
    ),
  ];

  for module in BUILTIN_MODULES.iter() {
    let filename = module.0;
    let source = module.1;

    let tc_scope = &mut v8::TryCatch::new(scope);

    let module = fetch_module(tc_scope, filename, Some(source)).unwrap();
    let _ = module
      .instantiate_module(tc_scope, module_resolve_cb)
      .unwrap();
    let _ = module.evaluate(tc_scope);

    if module.get_status() == v8::ModuleStatus::Errored {
      let exception = module.get_exception();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      error!(
        "Failed to evaluate builtin modules: {filename}, error: {exception:?}"
      );
      std::process::exit(1);
    }
  }
}

/// The state for js runtime of snapshot.
pub struct JsRuntimeStateForSnapshot {
  pub context: Option<v8::Global<v8::Context>>,
}

rc_refcell_ptr!(JsRuntimeStateForSnapshot);

/// The js runtime for snapshot.
///
/// WARNING: When creating snapshot, do remember that the `__InternalRsvimGlobalObject` bindings
/// are not available, because most of the functions are related with outside of js runtime, i.e.
/// the UI tree, the event loop, the tokio channels, etc. We cannot really make snapshot for them.
/// So when creating snapshot, we are mainly serialize those built-in modules, i.e. compile the
/// scripts into `v8::Module`.
///
/// TODO: Can we also evaluate these built-in modules (to further improve startup performance)?
pub struct JsRuntimeForSnapshot {
  /// V8 isolate.
  /// This is an `Option<v8::OwnedIsolate>` instead of just `v8::OwnedIsolate` is to workaround the
  /// safety issue with snapshot_creator.
  /// See: <https://github.com/denoland/deno/blob/d0efd040c79021958a1e83caa56572c0401ca1f2/core/runtime.rs?plain=1#L93>.
  pub isolate: Option<v8::OwnedIsolate>,

  /// State.
  pub state: JsRuntimeStateForSnapshotRc,
}

impl Drop for JsRuntimeForSnapshot {
  fn drop(&mut self) {
    debug_assert_eq!(Rc::strong_count(&self.state), 1);
  }
}

impl JsRuntimeForSnapshot {
  /// Creates a new js runtime for snapshot.
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    init_v8_platform(true, None);

    let (mut isolate, global_context) = Self::create_isolate();

    let mut context_scope =
      v8::HandleScope::with_context(&mut isolate, global_context.clone());
    let scope = &mut context_scope;
    // let _context = v8::Local::new(scope, global_context.clone());

    // Load, compile and evaluate all built-in modules.
    init_builtin_modules(scope);

    let state = JsRuntimeStateForSnapshot::to_rc(JsRuntimeStateForSnapshot {
      context: Some(global_context),
    });

    scope.set_slot(state.clone());

    drop(context_scope);

    JsRuntimeForSnapshot {
      isolate: Some(isolate),
      state,
    }
  }

  fn create_isolate() -> (v8::OwnedIsolate, v8::Global<v8::Context>) {
    let mut isolate =
      v8::Isolate::snapshot_creator(None, Some(v8::CreateParams::default()));

    init_v8_isolate(&mut isolate);

    let global_context = {
      let scope = &mut v8::HandleScope::new(&mut isolate);
      let context = v8::Context::new(scope, Default::default());
      v8::Global::new(scope, context)
    };

    (isolate, global_context)
  }

  pub fn create_snapshot(mut self) -> v8::StartupData {
    // Set default context
    {
      let global_context = self.context();
      let mut scope = self.handle_scope();
      let context = v8::Local::new(&mut scope, global_context);
      scope.set_default_context(context);
    }

    // Drop state (and the global context inside)
    {
      let state = self.get_state();
      state.borrow_mut().context.take();
    }

    let snapshot_creator = self.isolate.take().unwrap();
    snapshot_creator
      .create_blob(v8::FunctionCodeHandling::Keep)
      .unwrap()
  }
}

impl JsRuntimeForSnapshot {
  pub fn context(&self) -> v8::Global<v8::Context> {
    self.get_state().borrow().context.as_ref().unwrap().clone()
  }

  pub fn state(isolate: &v8::Isolate) -> JsRuntimeStateForSnapshotRc {
    isolate
      .get_slot::<JsRuntimeStateForSnapshotRc>()
      .unwrap()
      .clone()
  }

  pub fn get_state(&self) -> JsRuntimeStateForSnapshotRc {
    Self::state(self.isolate.as_ref().unwrap())
  }

  pub fn handle_scope(&mut self) -> v8::HandleScope<'_> {
    let context = self.context();
    v8::HandleScope::with_context(self.isolate.as_mut().unwrap(), context)
  }
}

#[derive(Debug, Default, Clone)]
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

/// The state of js runtime.
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
  // Sender: js runtime send to master.
  pub jsrt_to_mstr: Sender<JsRuntimeToEventLoopMessage>,
  // Receiver: js runtime receive from master.
  pub jsrt_from_mstr: Receiver<EventLoopToJsRuntimeMessage>,
  pub cli_opt: CliOpt,
  pub tree: TreeArc,
  pub buffers: BuffersManagerArc,
  pub contents: TextContentsArc,
  // Same as the `state` in EventLoop.
  pub editing_state: StateArc,
  // Data Access for RSVIM }
}

rc_refcell_ptr!(JsRuntimeState);

/// Snapshot data for startup.
pub struct SnapshotData {
  pub value: &'static [u8],
}

impl SnapshotData {
  pub fn new(value: &'static [u8]) -> Self {
    SnapshotData { value }
  }
}

/// Javascript runtime.
///
/// There are 3 most important concepts:
///
/// - Isolate
/// - Context
/// - Handle Scope
///
/// For more details, please see: <https://v8.dev/docs/embed>.
pub struct JsRuntime {
  /// V8 isolate.
  pub isolate: v8::OwnedIsolate,

  /// The state of the runtime.
  #[allow(unused)]
  pub state: JsRuntimeStateRc,
}

impl std::fmt::Debug for JsRuntime {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "JsRuntime")
  }
}

impl JsRuntime {
  /// Creates a new JsRuntime with snapshot.
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    options: JsRuntimeOptions,
    snapshot: SnapshotData,
    startup_moment: Instant,
    time_origin: u128,
    jsrt_to_mstr: Sender<JsRuntimeToEventLoopMessage>,
    jsrt_from_mstr: Receiver<EventLoopToJsRuntimeMessage>,
    cli_opt: CliOpt,
    tree: TreeArc,
    buffers: BuffersManagerArc,
    contents: TextContentsArc,
    editing_state: StateArc,
  ) -> Self {
    // Fire up the v8 engine.
    init_v8_platform(false, Some(&options.v8_flags));

    let mut isolate = {
      let create_params = v8::CreateParams::default();
      let create_params = create_params.snapshot_blob(snapshot.value.into());
      v8::Isolate::new(create_params)
    };

    init_v8_isolate(&mut isolate);

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

    // // Get snapshotted built-in modules data from context
    // fn get_context_data(
    //   scope: &mut v8::HandleScope<()>,
    //   context: v8::Local<v8::Context>,
    // ) -> Vec<v8::Global<v8::Module>> {
    //   fn data_error_to_panic(err: v8::DataError) -> ! {
    //     match err {
    //       v8::DataError::BadType { actual, expected } => {
    //         panic!("Invalid type for snapshot data: expected {expected}, got {actual}");
    //       }
    //       v8::DataError::NoData { expected } => {
    //         panic!("No data for snapshot data: expected {expected}");
    //       }
    //     }
    //   }
    //
    //   let mut scope = v8::ContextScope::new(scope, context);
    //
    //   let mut module_handles: Vec<v8::Global<v8::Module>> = vec![];
    //   for i in 0..BUILTIN_MODULES_LEN {
    //     match scope.get_context_data_from_snapshot_once::<v8::Module>(i) {
    //       Ok(val) => {
    //         let module_global = v8::Global::new(&mut scope, val);
    //         module_handles.push(module_global);
    //       }
    //       Err(err) => data_error_to_panic(err),
    //     }
    //   }
    //
    //   module_handles
    // }

    let context: v8::Global<v8::Context> = {
      let scope = &mut v8::HandleScope::new(&mut *isolate);
      let context = binding::create_new_context(scope);

      // let module_handles = get_context_data(scope, context);
      v8::Global::new(scope, context)
    };

    // Store state inside the v8 isolate slot.
    // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
    let state = JsRuntimeState::to_rc(JsRuntimeState {
      context,
      module_map: ModuleMap::new(),
      timeout_handles: HashSet::new(),
      // interrupt_handle: event_loop.interrupt_handle(),
      pending_futures: HashMap::new(),
      // timeout_queue: BTreeMap::new(),
      startup_moment,
      time_origin,
      // next_tick_queue: Vec::new(),
      exceptions: ExceptionState::new(),
      options,
      // wake_event_queued: false,
      jsrt_to_mstr,
      jsrt_from_mstr,
      cli_opt,
      tree,
      buffers,
      contents,
      editing_state,
    });

    isolate.set_slot(state.clone());

    JsRuntime {
      isolate,
      // event_loop,
      state,
      // inspector,
    }

    // With snapshot, we no longer need to initialize builtin runtime modules any more.
    // runtime.init_environment(module_handles);

    // // Start inspector agent is requested.
    // if let Some(inspector) = runtime.inspector().as_mut() {
    //   let address = address.unwrap();
    //   inspector.borrow_mut().start_agent(address);
    // }

    // runtime
  }

  /// Creates a new JsRuntime from scratch, i.e. without snapshot, usually for
  /// testing purpose.
  #[allow(clippy::too_many_arguments)]
  pub fn new_without_snapshot(
    options: JsRuntimeOptions,
    startup_moment: Instant,
    time_origin: u128,
    jsrt_to_mstr: Sender<JsRuntimeToEventLoopMessage>,
    jsrt_from_mstr: Receiver<EventLoopToJsRuntimeMessage>,
    cli_opt: CliOpt,
    tree: TreeArc,
    buffers: BuffersManagerArc,
    contents: TextContentsArc,
    editing_state: StateArc,
  ) -> Self {
    // Fire up the v8 engine.
    init_v8_platform(false, Some(&options.v8_flags));

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());

    init_v8_isolate(&mut isolate);

    let context: v8::Global<v8::Context> = {
      let scope = &mut v8::HandleScope::new(&mut *isolate);
      let context = binding::create_new_context(scope);

      // let module_handles = get_context_data(scope, context);
      v8::Global::new(scope, context)
    };

    // Store state inside the v8 isolate slot.
    // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
    let state = JsRuntimeState::to_rc(JsRuntimeState {
      context,
      module_map: ModuleMap::new(),
      timeout_handles: HashSet::new(),
      // interrupt_handle: event_loop.interrupt_handle(),
      pending_futures: HashMap::new(),
      // timeout_queue: BTreeMap::new(),
      startup_moment,
      time_origin,
      // next_tick_queue: Vec::new(),
      exceptions: ExceptionState::new(),
      options,
      // wake_event_queued: false,
      jsrt_to_mstr,
      jsrt_from_mstr,
      cli_opt,
      tree,
      buffers,
      contents,
      editing_state,
    });

    isolate.set_slot(state.clone());

    let mut runtime = JsRuntime {
      isolate,
      // event_loop,
      state,
      // inspector,
    };

    // When without snapshot, we need to initialize builtin js modules.
    init_builtin_modules(&mut runtime.handle_scope());

    // // Start inspector agent is requested.
    // if let Some(inspector) = runtime.inspector().as_mut() {
    //   let address = address.unwrap();
    //   inspector.borrow_mut().start_agent(address);
    // }

    runtime
  }

  // /// Executes traditional JavaScript code (traditional = not ES modules).
  // ///
  // /// NOTE: We don't use it.
  // pub fn __execute_script(
  //   &mut self,
  //   filename: &str,
  //   source: &str,
  // ) -> Result<Option<v8::Global<v8::Value>>, AnyErr> {
  //   // Get the handle-scope.
  //   let scope = &mut self.handle_scope();
  //   let state_rc = JsRuntime::state(scope);
  //
  //   let origin = create_origin(scope, filename, false);
  //   let source = v8::String::new(scope, source).unwrap();
  //
  //   // The `TryCatch` scope allows us to catch runtime errors rather than panicking.
  //   let tc_scope = &mut v8::TryCatch::new(scope);
  //   type ExecuteScriptResult = Result<Option<v8::Global<v8::Value>>, AnyErr>;
  //
  //   let handle_exception = |scope: &mut v8::TryCatch<
  //     '_,
  //     v8::HandleScope<'_>,
  //   >|
  //    -> ExecuteScriptResult {
  //     // Extract the exception during compilation.
  //     assert!(scope.has_caught());
  //     let exception = scope.exception().unwrap();
  //     let exception = v8::Global::new(scope, exception);
  //     let mut state = state_rc.borrow_mut();
  //     // Capture the exception internally.
  //     state.exceptions.capture_exception(exception);
  //     drop(state);
  //     // Force an exception check.
  //     if let Some(error) = check_exceptions(scope) {
  //       anyhow::bail!(error)
  //     }
  //     Ok(None)
  //   };
  //
  //   let script = match v8::Script::compile(tc_scope, source, Some(&origin)) {
  //     Some(script) => script,
  //     None => return handle_exception(tc_scope),
  //   };
  //
  //   match script.run(tc_scope) {
  //     Some(value) => Ok(Some(v8::Global::new(tc_scope, value))),
  //     None => handle_exception(tc_scope),
  //   }
  // }

  /// Executes JavaScript code as ES module.
  pub fn execute_module(
    &mut self,
    filename: &str,
    source: Option<&str>,
  ) -> Result<(), AnyErr> {
    // Get a reference to v8's scope.
    let scope = &mut self.handle_scope();

    // The following code allows the runtime to execute code with no valid
    // location passed as parameter as an ES module.
    let path = if source.is_some() {
      filename.to_string()
    } else {
      match resolve_import(None, filename, None) {
        Ok(specifier) => specifier,
        Err(e) => {
          // Returns the error directly.
          return Err(e);
        }
      }
    };
    trace!("Resolved main js module (path): {:?}", path);

    let tc_scope = &mut v8::TryCatch::new(scope);

    // NOTE: Here we also use static module fetching, i.e. all the modules are already stored on
    // local file system, no network/http downloading will be involved.
    let module = match fetch_module_tree(tc_scope, filename, None) {
      Some(module) => module,
      None => {
        assert!(tc_scope.has_caught());
        let exception = tc_scope.exception().unwrap();
        let _exception = JsError::from_v8_exception(tc_scope, exception, None);
        let e = format!("User config not found: {filename:?}");
        error!(e);
        eprintln!("{e}");
        anyhow::bail!(e);
      }
    };

    if module
      .instantiate_module(tc_scope, module_resolve_cb)
      .is_none()
    {
      assert!(tc_scope.has_caught());
      let exception = tc_scope.exception().unwrap();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      let e = format!(
        "Failed to instantiate user config module {filename:?}: {exception:?}"
      );
      error!(e);
      eprintln!("{e}");
      anyhow::bail!(e);
    }

    match module.evaluate(tc_scope) {
      Some(result) => {
        trace!(
          "Evaluated user config module result ({:?}): {:?}",
          result.type_repr(),
          result.to_rust_string_lossy(tc_scope),
        );
      }
      None => trace!("Evaluated user config module result: None"),
    }

    if module.get_status() == v8::ModuleStatus::Errored {
      let exception = module.get_exception();
      let exception = JsError::from_v8_exception(tc_scope, exception, None);
      let e = format!(
        "Failed to evaluate user config module {filename:?}: {exception:?}"
      );
      error!(e);
      eprintln!("{e}");
      anyhow::bail!(e);
    }

    Ok(())
  }

  /// Runs a single tick of the event-loop.
  pub fn tick_event_loop(&mut self) {
    let isolate_has_pending_tasks = self.isolate.has_pending_background_tasks();
    trace!(
      "Tick js runtime, isolate has pending tasks: {:?}",
      isolate_has_pending_tasks
    );
    run_next_tick_callbacks(&mut self.handle_scope());
    self.fast_forward_imports();
    // self.event_loop.tick();
    self.run_pending_futures();
    trace!("Tick js runtime - done");
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
      while let Ok(msg) = state.jsrt_from_mstr.try_recv() {
        match msg {
          EventLoopToJsRuntimeMessage::TimeoutResp(resp) => {
            trace!("Receive TimeResp:{resp:?}");
            match state.pending_futures.remove(&resp.future_id) {
              Some(timeout_cb) => futures.push(timeout_cb),
              None => unreachable!(
                "Failed to get timeout future by ID {:?}",
                resp.future_id
              ),
            }
          }
          EventLoopToJsRuntimeMessage::ExCommandReq(req) => {
            trace!("Receive ExCommandReq:{req:?}");
            debug_assert!(!state.pending_futures.contains_key(&req.future_id));
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
    let mut ready_imports = vec![];

    {
      let state = state_rc.borrow();

      let mut seen_modules = state.module_map.seen().borrow_mut();
      let mut pending_graphs = state.module_map.pending().borrow_mut();

      pending_graphs.retain(|graph_rc| {
        // Get a usable ref to graph's root module.
        let graph = graph_rc.borrow();
        let graph_root = graph.root_rc();
        let mut graph_root = graph_root.borrow_mut();

        // Check for exceptions in the graph (dynamic imports).
        if let Some(message) = graph_root.exception_mut().take() {
          // Create a v8 exception.
          let exception = v8::String::new(scope, &message).unwrap();
          let exception = v8::Exception::error(scope, exception);

          // We need to resolve all identical dynamic imports.
          match graph.kind().clone() {
            ImportKind::Static => unreachable!(),
            ImportKind::Dynamic(main_promise) => {
              for promise in
                [main_promise].iter().chain(graph.same_origin().iter())
              {
                promise.open(scope).reject(scope, exception);
              }
            }
          }

          return false;
        }

        // If the graph is still loading, fast-forward the dependencies.
        if graph_root.status() != ModuleStatus::Ready {
          graph_root.fast_forward(&mut seen_modules);
          return true;
        }

        ready_imports.push(Rc::clone(graph_rc));
        false
      });

      // Note: We have to drop the sate ref here to avoid borrow panics
      // during the module instantiation/evaluation process.
      // drop(state);
    }

    // Execute the root module from the graph.
    for graph_rc in ready_imports {
      // Create a tc scope.
      let tc_scope = &mut v8::TryCatch::new(scope);

      let graph = graph_rc.borrow();
      let path = graph.root_rc().borrow().path().clone();

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
      let is_root_module = !graph.root_rc().borrow().is_dynamic_import();

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

      if let ImportKind::Dynamic(main_promise) = graph.kind().clone() {
        // Note: Since this is a dynamic import will resolve the promise
        // with the module's namespace object instead of it's evaluation result.
        let namespace = module.get_module_namespace();

        // We need to resolve all identical dynamic imports.
        for promise in [main_promise].iter().chain(graph.same_origin().iter()) {
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
    !self
      .get_state()
      .borrow()
      .module_map
      .pending()
      .borrow()
      .is_empty()
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
  pub fn state(isolate: &v8::Isolate) -> JsRuntimeStateRc {
    isolate.get_slot::<JsRuntimeStateRc>().unwrap().clone()
  }

  /// Returns the runtime's state.
  pub fn get_state(&self) -> JsRuntimeStateRc {
    Self::state(&self.isolate)
  }

  /// Returns a v8 handle scope for the runtime.
  /// See: <https://v8docs.nodesource.com/node-0.8/d3/d95/classv8_1_1_handle_scope.html>.
  pub fn handle_scope(&mut self) -> v8::HandleScope<'_> {
    let context = self.context();
    v8::HandleScope::with_context(&mut self.isolate, context)
  }

  /// Returns a global context created for the runtime.
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
