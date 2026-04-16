//! Syntax APIs.

pub mod load;

use crate::js;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use crate::syntax;
use crate::syntax::SyntaxLoadGrammarRequest;
pub use load::SynLoadTreeSitterGrammarFuture;
pub use load::SynLoadTreeSitterGrammarOptions;
use std::path::Path;

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

  let load_req = SyntaxLoadGrammarRequest {
    grammar_path: Path::new(&options.grammar_path).to_path_buf(),
  };
  match syntax::load_syntax_grammar(state.syntax_manager.clone(), &load_req) {
    Ok(metainfo) => {
      let grammar_names = metainfo
        .grammars
        .iter()
        .map(|gm| gm.name.to_string())
        .collect::<Vec<String>>()
        .to_v8(scope, |scope, grammar_name| {
          grammar_name.to_v8(scope).into()
        });
      rv.set(grammar_names.into());
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
  pending::create_syn_load_treesitter_grammar(
    &mut state,
    task_id,
    grammar_path,
    Box::new(load_cb),
  );

  rv.set(promise.into());
}
