//! Load tree-sitter grammar APIs.

use crate::from_v8_prop;
use crate::is_v8_bool;
use crate::is_v8_int;
use crate::js;
use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::TimerId;
use crate::js::binding;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use crate::to_v8_prop;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::rc::Rc;

pub const GRAMMAR_PATH: &str = "grammarPath";
pub const OUTPUT_PATH: &str = "outputPath";

pub const GRAMMAR_PATH_DEFAULT: &str = "";
pub const OUTPUT_PATH_DEFAULT: &str = "";

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct SynLoadTreeSitterGrammarOptions {
  #[builder(default = GRAMMAR_PATH_DEFAULT.to_compact_string())]
  pub grammar_path: CompactString,

  #[builder(default = OUTPUT_PATH_DEFAULT.to_compact_string())]
  pub output_path: CompactString,
}

impl StructFromV8 for SynLoadTreeSitterGrammarOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = SynLoadTreeSitterGrammarOptionsBuilder::default();

    from_v8_prop!(builder, obj, scope, CompactString, grammar_path);
    from_v8_prop!(builder, obj, scope, CompactString, output_path);

    builder.build().unwrap()
  }
}

impl StructToV8 for SynLoadTreeSitterGrammarOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, grammar_path);
    to_v8_prop!(self, obj, scope, output_path);

    obj
  }
}

struct LoadTreeSitterGrammarFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for LoadTreeSitterGrammarFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|LoadTreeSitterGrammarFuture|");

    let result = self.maybe_result.take().unwrap();

    // Handle when something goes wrong with opening the file.
    if let Err(e) = result {
      let message = v8::String::new(scope, &e.to_string()).unwrap();
      let exception = v8::Exception::error(scope, message);
      binding::set_exception_code(scope, exception, &e);
      self.promise.open(scope).reject(scope, exception);
      return;
    }

    // Otherwise, get the result and deserialize it.
    let result = result.unwrap();

    // Deserialize bytes into a loaded grammar name.
    let grammar_id = postcard::from_bytes::<String>(&result).unwrap();
    let grammar_id = grammar_id.to_v8(scope);

    self
      .promise
      .open(scope)
      .resolve(scope, grammar_id.into())
      .unwrap();
  }
}

/// Javascript `loadTreeSitterGrammarSync` API.
pub fn load_treesitter_grammar<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let options =
    FsOpenOptions::from_v8(scope, args.get(1).to_object(scope).unwrap());
  let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
  let callback = Rc::new(v8::Global::new(scope, callback));

  // Get timer's delay time in millis.
  debug_assert!(is_v8_int!(args.get(1)));
  let delay = u32::from_v8(scope, args.get(1).to_integer(scope).unwrap());
  // Get timer's repeated.
  debug_assert!(is_v8_bool!(args.get(2)));
  let repeated = bool::from_v8(scope, args.get(2).to_boolean(scope));

  // NOTE: Don't delete this part of code, it shows how to convert function
  // arguments into an array of values.
  //
  // Convert params argument (Array<Local<Value>>) to Rust vector.
  // let params = match v8::Local::<v8::Array>::try_from(args.get(3)) {
  //   Ok(params) => (0..params.length()).fold(
  //     Vec::<v8::Global<v8::Value>>::new(),
  //     |mut acc, i| {
  //       let param = params.get_index(scope, i).unwrap();
  //       acc.push(v8::Global::new(scope, param));
  //       acc
  //     },
  //   ),
  //   Err(_) => vec![],
  // };

  // NOTE: Since in javascript side, we don't pass any extra parameters to
  // timers, thus it is always empty array. But, we leave this code here as a
  // reference.
  let params = vec![];
  let params = Rc::new(params);

  let state_rc = JsRuntime::state(scope);
  let timer_cb = {
    let state_rc = state_rc.clone();
    move || {
      let fut = TimeoutFuture {
        cb: Rc::clone(&callback),
        params: Rc::clone(&params),
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.push(Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let timer_id = js::TimerId::next();
  pending::create_timer(
    &mut state,
    timer_id,
    delay,
    repeated,
    Box::new(timer_cb),
  );
  rv.set_int32(timer_id.into());
  trace!(
    "|create_timer| timer_id:{:?}, delay:{:?}, repeated:{:?}",
    timer_id, delay, repeated
  );
}

/// Javascript `clearTimeout`/`clearInterval` API.
pub fn clear_timer<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  // Get timer ID, and remove it.
  debug_assert!(is_v8_int!(args.get(0)));
  let timer_id =
    TimerId::from_v8(scope, args.get(0).to_integer(scope).unwrap());
  let state_rc = JsRuntime::state(scope);

  let mut state = state_rc.borrow_mut();
  pending::remove_timer(&mut state, timer_id);
  trace!("|clear_timer| timer_id:{:?}", timer_id);
}
