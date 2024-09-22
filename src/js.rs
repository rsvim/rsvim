//! JavaScript runtime.

#![allow(dead_code, unused)]

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, error};

use crate::buf::BuffersArc;
use crate::glovar;
use crate::js::module::transpiler::transpile_extension;
use crate::js::msg::{EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage};
use crate::result::{ErrorCode, VoidResult};
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

pub mod module;
pub mod msg;

pub async fn start(data_access: JsDataAccess) -> VoidResult {
  if let Some(config_entry) = glovar::CONFIG_FILE_PATH() {
    debug!("Read config entry: {:?}", config_entry);
    let cwd = std::env::current_dir().unwrap();
    let main_module = deno_core::resolve_path(config_entry, cwd.as_path()).unwrap();
    let mut deno_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
      module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
      extension_transpiler: Some(Rc::new(|specifier, code| {
        transpile_extension(&specifier, &code)
      })),
      ..Default::default()
    });
    deno_runtime
      .execute_script("[vim:runtime.js]", include_str!("./runtime.js"))
      .unwrap();

    let main_module_id = deno_runtime
      .load_main_es_module(&main_module)
      .await
      .unwrap();
    debug!("Load main module id: {:?}", main_module_id);
    let evaluate_result = deno_runtime.mod_evaluate(main_module_id);
    let run_evloop_result = deno_runtime
      .run_event_loop(deno_core::PollEventLoopOptions::default())
      .await;
    debug!(
      "Run event loop on main module result: {:?}",
      run_evloop_result
    );
    let result = evaluate_result.await;
    debug!("Evaluate main module result: {:?}", result);
  }

  Ok(())
}

#[derive(Debug)]
/// The mutable data passed to each state handler, and allow them access the editor.
pub struct JsDataAccess {
  pub js_send_to_evloop: Sender<JsRuntimeToEventLoopMessage>,
  pub js_recv_from_evloop: Receiver<EventLoopToJsRuntimeMessage>,

  pub state: StateArc,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
}

impl JsDataAccess {
  pub fn new(
    js_send_to_evloop: Sender<JsRuntimeToEventLoopMessage>,
    js_recv_from_evloop: Receiver<EventLoopToJsRuntimeMessage>,
    state: StateArc,
    tree: TreeArc,
    buffers: BuffersArc,
  ) -> Self {
    JsDataAccess {
      js_send_to_evloop,
      js_recv_from_evloop,
      state,
      tree,
      buffers,
    }
  }
}
