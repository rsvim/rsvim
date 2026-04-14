//! Syntax APIs.

pub mod load;

use crate::js;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use crate::syntax::SyntaxLoadGrammarRequest;
use crate::syntax::load_grammar;
pub use load::SynLoadTreeSitterGrammarFuture;
pub use load::SynLoadTreeSitterGrammarOptions;

/// Javascript `loadTreeSitterGrammarSync` API.
pub fn load_treesitter_grammar_sync<'s>(
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

/// Javascript `loadTreeSitterGrammar` API.
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

  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let load_cb = {
    let promise = v8::Global::new(scope, promise_resolver);
    let state_rc = state_rc.clone();
    move |maybe_result: Option<TheResult<Vec<u8>>>| {
      let fut = SynLoadTreeSitterGrammarFuture {
        promise: promise.clone(),
        maybe_result,
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.push(Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let task_id = js::TaskId::next();
  let grammar_path = Path::new(&options.grammar_path);
  let output_path = Path::new(&options.output_path);
  pending::create_syn_load_treesitter_grammar(
    &mut state,
    task_id,
    grammar_path,
    output_path,
    Box::new(load_cb),
  );

  rv.set(promise.into());
}
