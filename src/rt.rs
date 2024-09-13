//! The JavaScript runtime.

#![allow(dead_code)]

use std::sync::Once;
use std::time::Duration;
use tokio::fs;
use tracing::{debug, error};
use v8::MapFnTo;

use crate::buf::BuffersArc;
use crate::glovar;
use crate::result::{ErrorCode, VoidResult};
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

static INIT: Once = Once::new();

fn into_str(buf: &[u8], bufsize: usize) -> String {
  String::from_utf8_lossy(&buf[0..bufsize]).into_owned()
}

fn init_v8_platform() {
  let platform = v8::new_default_platform(0, false).make_shared();
  v8::V8::initialize_platform(platform);
  v8::V8::initialize();
}

pub struct JsRuntime {
  isolate: v8::OwnedIsolate,
  config_file: String,
}

impl JsRuntime {
  pub fn new(config_file: String) -> Self {
    INIT.call_once(init_v8_platform);
    let isolate = v8::Isolate::new(Default::default());

    JsRuntime {
      isolate,
      config_file,
    }
  }

  pub async fn start(&mut self, mut data_access: JsDataAccess) -> VoidResult {
    let scope = &mut v8::HandleScope::new(&mut self.isolate);

    // Create the `vim` global object {

    let global_vim_template = v8::ObjectTemplate::new(scope);
    let mut accessor_property = v8::PropertyAttribute::NONE;
    accessor_property = accessor_property | v8::PropertyAttribute::READ_ONLY;

    let line_wrap_getter = {
      let external = v8::External::new(
        scope,
        CallbackInfo::new_raw((&mut data_access) as *mut JsDataAccess) as *mut _,
      );
      let function = v8::FunctionTemplate::builder_raw(line_wrap_getter_call_fn)
        .data(external.into())
        .build(scope);

      if let Some(v8str) = v8::String::new(scope, "getLineWrap").unwrap().into() {
        function.set_class_name(v8str);
      }

      function
    };

    let line_wrap_setter = {
      let external = v8::External::new(
        scope,
        CallbackInfo::new_raw((&mut data_access) as *mut JsDataAccess) as *mut _,
      );
      let function = v8::FunctionTemplate::builder_raw(line_wrap_setter_call_fn)
        .data(external.into())
        .build(scope);

      if let Some(v8str) = v8::String::new(scope, "setLineWrap").unwrap().into() {
        function.set_class_name(v8str);
      }

      function
    };

    global_vim_template.set_accessor_property(
      v8::String::new(scope, "vim").unwrap().into(),
      Some(line_wrap_getter),
      Some(line_wrap_setter),
      accessor_property,
    );

    // Create the `vim` global object }

    let context = v8::Context::new(scope, Default::default());
    let _scope = &mut v8::ContextScope::new(scope, context);

    debug!("Load config file {:?}", self.config_file.as_str());
    match fs::read_to_string(self.config_file.as_str()).await {
      Ok(source) => {}
      Err(e) => {
        let msg = format!(
          "Failed to load user config file {:?} with error {:?}",
          self.config_file.as_str(),
          e
        );
        error!("{msg}");
        return Err(ErrorCode::Message(msg));
      }
    }

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

#[repr(C)]
#[derive(Debug)]
pub struct CallbackInfo {
  pub env: *mut JsDataAccess,
}

impl CallbackInfo {
  #[inline]
  pub fn new_raw(env: *mut JsDataAccess) -> *mut Self {
    Box::into_raw(Box::new(Self { env }))
  }
}

pub fn create_function_template<'s>(
  scope: &mut v8::HandleScope<'s>,
  env: *mut JsDataAccess,
  name: Option<v8::Local<v8::String>>,
) -> v8::Local<'s, v8::FunctionTemplate> {
  let external = v8::External::new(scope, CallbackInfo::new_raw(env) as *mut _);
  let function = v8::FunctionTemplate::builder_raw(line_wrap_getter_call_fn)
    .data(external.into())
    .build(scope);

  if let Some(v8str) = name {
    function.set_class_name(v8str);
  }

  function
}

// fn create_function<'s>(
//   scope: &mut v8::HandleScope<'s>,
//   name: Option<v8::Local<v8::String>>,
//   data_access: *mut JsDataAccess,
// ) -> v8::Local<'s, v8::Function> {
//   let external = v8::External::new(scope, CallbackInfo::new_raw(data_access) as *mut _);
//   let function = v8::Function::builder_raw(line_wrap_getter_call_fn)
//     .data(external.into())
//     .build(scope)
//     .unwrap();
//
//   if let Some(v8str) = name {
//     function.set_name(v8str);
//   }
//
//   function
// }

extern "C" fn line_wrap_getter_call_fn(info: *const v8::FunctionCallbackInfo) {
  let callback_info = unsafe { &*info };
  let args = v8::FunctionCallbackArguments::from_function_callback_info(callback_info);
  let mut rv = v8::ReturnValue::from_function_callback_info(callback_info);

  unsafe {
    // SAFETY: create_function guarantees that the data is a CallbackInfo external.
    let info_ptr: *mut CallbackInfo = {
      let external_value = v8::Local::<v8::External>::cast_unchecked(args.data());
      external_value.value() as _
    };

    // SAFETY: pointer from Box::into_raw.
    let info = &mut *info_ptr;

    rv.set_bool(
      std::ptr::NonNull::new(info.env)
        .unwrap()
        .as_ref()
        .tree
        .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .line_wrap(),
    );
  }
}

extern "C" fn line_wrap_setter_call_fn(info: *const v8::FunctionCallbackInfo) {
  let callback_info = unsafe { &*info };
  let args = v8::FunctionCallbackArguments::from_function_callback_info(callback_info);
  let mut rv = v8::ReturnValue::from_function_callback_info(callback_info);

  unsafe {
    // SAFETY: create_function guarantees that the data is a CallbackInfo external.
    let info_ptr: *mut CallbackInfo = {
      let external_value = v8::Local::<v8::External>::cast_unchecked(args.data());
      external_value.value() as _
    };

    // SAFETY: pointer from Box::into_raw.
    let info = &mut *info_ptr;

    if args.length() == 1 && args.get(0).is_boolean() {
      let value = args.get(0).is_true();
      std::ptr::NonNull::new(info.env)
        .unwrap()
        .as_ref()
        .tree
        .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .set_line_wrap(value);
    }

    rv.set_undefined();
  }
}
