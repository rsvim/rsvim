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
use crate::js_runtime::module::transpiler::transpile_extension;
use crate::js_runtime::msg::{EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage};
use crate::result::{ErrorCode, VoidResult};
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

pub mod module;
pub mod msg;
pub mod path_config;

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

  pub fn start(&mut self, _data_access: JsDataAccess) -> VoidResult {
    let deno_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
      module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
      extension_transpiler: Some(Rc::new(|specifier, code| {
        transpile_extension(&specifier, &code)
      })),
      ..Default::default()
    });

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
