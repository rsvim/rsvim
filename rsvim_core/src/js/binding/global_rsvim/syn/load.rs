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
use crate::syntax::SyntaxLoadGrammarRequest;
use crate::syntax::async_load_grammar;
use crate::syntax::load_grammar;
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
  let options = SynLoadTreeSitterGrammarOptions::from_v8(
    scope,
    args.get(0).to_object(scope).unwrap(),
  );
  trace!("Rsvim.syn.loadTreeSitterGrammarSync:{:?}", options);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();
  let syn_loader = lock!(state.syntax_manager).loader();
  let req = SyntaxLoadGrammarRequest {
    grammar_path: Path::new(&options.grammar_path).to_path_buf(),
    output_path: Path::new(&options.output_path).to_path_buf(),
  };

  match load_grammar(syn_loader, req) {
    Ok(grammar_id) => {
      rv.set(v8::String::new(scope, &grammar_id).unwrap().into());
    }
    Err(e) => {
      binding::throw_exception(scope, &e);
    }
  }
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
