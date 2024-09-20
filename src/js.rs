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

pub struct JsRuntime {
  js_send_to_evloop: Sender<JsRuntimeToEventLoopMessage>,
  js_recv_from_evloop: Receiver<EventLoopToJsRuntimeMessage>,
}

impl JsRuntime {
  pub fn new(
    js_send_to_evloop: Sender<JsRuntimeToEventLoopMessage>,
    js_recv_from_evloop: Receiver<EventLoopToJsRuntimeMessage>,
  ) -> Self {
    JsRuntime {
      js_send_to_evloop,
      js_recv_from_evloop,
    }
  }

  pub async fn start(&mut self, data_access: JsDataAccess) -> VoidResult {
    let path_config = {
      data_access
        .state
        .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .path_config()
        .clone()
    };

    if let Some(config_entry) = path_config.config_file() {
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

    // debug!("Load config file {:?}", config_file.as_str());
    // match std::fs::read_to_string(config_file.as_str()) {
    //   Ok(source) => {
    //     debug!("Load source code:{:?}", source.as_str());
    //     let code = v8::String::new(scope, source.as_str()).unwrap();
    //     let script = v8::Script::compile(scope, code, None).unwrap();
    //     let result = script.run(scope).unwrap();
    //     let result = result.to_string(scope).unwrap();
    //     debug!("Execute result: {}", result.to_rust_string_lossy(scope));
    //   }
    //   Err(e) => {
    //     let msg = format!(
    //       "Failed to load user config file {:?} with error {:?}",
    //       config_file.as_str(),
    //       e
    //     );
    //     error!("{msg}");
    //     return Err(ErrorCode::Message(msg));
    //   }
    // }

    Ok(())
  }
}

#[derive(Debug)]
/// The mutable data passed to each state handler, and allow them access the editor.
pub struct JsDataAccess {
  pub state: StateArc,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
}

impl JsDataAccess {
  pub fn new(state: StateArc, tree: TreeArc, buffers: BuffersArc) -> Self {
    JsDataAccess {
      state,
      tree,
      buffers,
    }
  }
}
