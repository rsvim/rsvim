//! JavaScript runtime.

#![allow(dead_code, unused)]

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Once;
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error};

// use crate::buf::BuffersArc;
// use crate::glovar;
use crate::js::module::{ImportMap, ModuleMap};
// use crate::js::msg::{EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage};
// use crate::result::VoidResult;
// use crate::state::StateArc;
// use crate::ui::tree::TreeArc;

pub mod binding;
pub mod constants;
pub mod hook;
pub mod loader;
pub mod module;
pub mod transpiler;

// pub async fn start(data_access: JsDataAccess) -> VoidResult {
//   if let Some(config_entry) = glovar::CONFIG_FILE_PATH() {
//     debug!("Read config entry: {:?}", config_entry);
//     let cwd = std::env::current_dir().unwrap();
//     let main_module = deno_core::resolve_path(config_entry, cwd.as_path()).unwrap();
//     let mut deno_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
//       module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
//       extension_transpiler: Some(Rc::new(|specifier, code| {
//         transpile_extension(&specifier, &code)
//       })),
//       ..Default::default()
//     });
//     deno_runtime
//       .execute_script("[vim:runtime.js]", include_str!("./runtime.js"))
//       .unwrap();
//
//     let main_module_id = deno_runtime
//       .load_main_es_module(&main_module)
//       .await
//       .unwrap();
//     debug!("Load main module id: {:?}", main_module_id);
//     let evaluate_result = deno_runtime.mod_evaluate(main_module_id);
//     let run_evloop_result = deno_runtime
//       .run_event_loop(deno_core::PollEventLoopOptions::default())
//       .await;
//     debug!(
//       "Run event loop on main module result: {:?}",
//       run_evloop_result
//     );
//     let result = evaluate_result.await;
//     debug!("Evaluate main module result: {:?}", result);
//   }
//
//   Ok(())
// }

// #[derive(Debug)]
// /// The mutable data passed to each state handler, and allow them access the editor.
// pub struct JsDataAccess {
//   pub js_send_to_evloop: Sender<JsRuntimeToEventLoopMessage>,
//   pub js_recv_from_evloop: Receiver<EventLoopToJsRuntimeMessage>,
//
//   pub state: StateArc,
//   pub tree: TreeArc,
//   pub buffers: BuffersArc,
// }
//
// impl JsDataAccess {
//   pub fn new(
//     js_send_to_evloop: Sender<JsRuntimeToEventLoopMessage>,
//     js_recv_from_evloop: Receiver<EventLoopToJsRuntimeMessage>,
//     state: StateArc,
//     tree: TreeArc,
//     buffers: BuffersArc,
//   ) -> Self {
//     JsDataAccess {
//       js_send_to_evloop,
//       js_recv_from_evloop,
//       state,
//       tree,
//       buffers,
//     }
//   }
// }

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
  // // Indicates if we're running JavaScript tests.
  // pub test_mode: bool,
  // // Defines the inspector listening options.
  // pub inspect: Option<(SocketAddrV4, bool)>,
  // // Exposes v8's garbage collector.
  // pub expose_gc: bool,
}

pub struct JsRuntimeState {
  /// A sand-boxed execution context with its own set of built-in objects and functions.
  pub context: v8::Global<v8::Context>,
  /// Holds information about resolved ES modules.
  pub module_map: ModuleMap,
  // /// A handle to the runtime's event-loop.
  // pub handle: LoopHandle,
  // /// A handle to the event-loop that can interrupt the poll-phase.
  // pub interrupt_handle: LoopInterruptHandle,
  // /// Holds JS pending futures scheduled by the event-loop.
  // pub pending_futures: Vec<Box<dyn JsFuture>>,
  /// Indicates the start time of the process.
  pub startup_moment: Instant,
  /// Specifies the timestamp which the current process began in Unix time.
  pub time_origin: u128,
  // /// Holds callbacks scheduled by nextTick.
  // pub next_tick_queue: NextTickQueue,
  // /// Stores and manages uncaught exceptions.
  // pub exceptions: ExceptionState,
  /// Runtime options.
  pub options: JsRuntimeOptions,
  // /// Tracks wake event for current loop iteration.
  // pub wake_event_queued: bool,
}

pub struct JsRuntime {
  // V8 isolate.
  isolate: v8::OwnedIsolate,

  /// The state of the runtime.
  #[allow(unused)]
  pub state: Rc<RefCell<JsRuntimeState>>,
}

impl JsRuntime {
  pub fn new() -> Self {
    Self::with_options(JsRuntimeOptions::default())
  }

  /// Creates a new JsRuntime based on provided options.
  pub fn with_options(options: JsRuntimeOptions) -> JsRuntime {
    // Configuration flags for V8.
    let mut flags = String::from(concat!(" --no-validate-asm", " --js-float16array",));

    v8::V8::set_flags_from_string(&flags);

    // Fire up the v8 engine.
    static V8_INIT: Once = Once::new();
    V8_INIT.call_once(move || {
      let platform = v8::new_default_platform(0, false).make_shared();
      v8::V8::initialize_platform(platform);
      v8::V8::initialize();
    });

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());

    isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
    isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);
    isolate.set_promise_reject_callback(hook::promise_reject_cb);
    isolate.set_host_import_module_dynamically_callback(host_import_module_dynamically_cb);
    isolate.set_host_initialize_import_meta_object_callback(host_initialize_import_meta_object_cb);

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

    let time_origin = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();

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
      // module_map: ModuleMap::new(),
      // handle: event_loop.handle(),
      // interrupt_handle: event_loop.interrupt_handle(),
      // pending_futures: Vec::new(),
      startup_moment: Instant::now(),
      time_origin,
      // next_tick_queue: Vec::new(),
      exceptions: ExceptionState::new(),
      options,
      // wake_event_queued: false,
    }));

    isolate.set_slot(state.clone());

    let mut runtime = JsRuntime {
      isolate,
      event_loop,
      state,
      inspector,
    };

    runtime.load_main_environment();

    // // Start inspector agent is requested.
    // if let Some(inspector) = runtime.inspector().as_mut() {
    //   let address = address.unwrap();
    //   inspector.borrow_mut().start_agent(address);
    // }

    runtime
  }

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
}
