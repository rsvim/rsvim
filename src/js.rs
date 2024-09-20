//! JavaScript runtime.

#![allow(dead_code, unused)]

use std::cell::RefCell;
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

pub mod bridge;
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

  pub fn start(&mut self, _data_access: JsDataAccess) -> VoidResult {
    let deno_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
      module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
      extension_transpiler: Some(Rc::new(|specifier, code| {
        transpile_extension(&specifier, &code)
      })),
      ..Default::default()
    });

    // let isolate = &mut v8::Isolate::new(Default::default());
    // let scope = &mut v8::HandleScope::new(isolate);

    // // Create the `vim` global object {
    //
    // let global_vim_template = v8::ObjectTemplate::new(scope);
    // let mut accessor_property = v8::PropertyAttribute::NONE;
    // accessor_property = accessor_property | v8::PropertyAttribute::READ_ONLY;
    //
    // let line_wrap_getter = {
    //   let external = v8::External::new(
    //     scope,
    //     CallbackInfo::new_raw((&mut data_access) as *mut JsDataAccess) as *mut _,
    //   );
    //   let function = v8::FunctionTemplate::builder_raw(line_wrap_getter_call_fn)
    //     .data(external.into())
    //     .build(scope);
    //
    //   if let Some(v8str) = v8::String::new(scope, "getLineWrap").unwrap().into() {
    //     function.set_class_name(v8str);
    //   }
    //
    //   function
    // };
    //
    // let line_wrap_setter = {
    //   let external = v8::External::new(
    //     scope,
    //     CallbackInfo::new_raw((&mut data_access) as *mut JsDataAccess) as *mut _,
    //   );
    //   let function = v8::FunctionTemplate::builder_raw(line_wrap_setter_call_fn)
    //     .data(external.into())
    //     .build(scope);
    //
    //   if let Some(v8str) = v8::String::new(scope, "setLineWrap").unwrap().into() {
    //     function.set_class_name(v8str);
    //   }
    //
    //   function
    // };
    //
    // global_vim_template.set_accessor_property(
    //   v8::String::new(scope, "vim").unwrap().into(),
    //   Some(line_wrap_getter),
    //   Some(line_wrap_setter),
    //   accessor_property,
    // );
    //
    // // Create the `vim` global object }

    // let context = v8::Context::new(scope, Default::default());
    // let scope = &mut v8::ContextScope::new(scope, context);
    // let config_file = ".rsvim.js".to_string();

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
