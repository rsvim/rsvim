//! Sub-process APIs.

use crate::js;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use crate::syntax;
use crate::syntax::SyntaxLoadGrammarRequest;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::ffi::OsStr;
use std::ffi::OsString;

#[derive(Debug, Clone)]
pub struct Command {
  pub program: OsString,
  pub args: Vec<OsString>,
  pub current_dir: Option<OsString>,
  pub envs: Vec<OsString>,
}

/// Javascript `loadParserSync` API.
pub fn load_parser_sync<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let options = SynLoadTreeSitterParserOptions::from_v8(
    scope,
    args.get(0).to_object(scope).unwrap(),
  );
  trace!("Rsvim.syn.loadParserSync:{:?}", options);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();

  match Path::new(&options.grammar_path).absolutize() {
    Ok(grammar_path) => {
      let load_req = SyntaxLoadGrammarRequest {
        grammar_path: grammar_path.to_path_buf(),
      };
      match syntax::load_syntax_grammar(state.syntax_manager.clone(), &load_req)
      {
        Ok(metainfo) => {
          let parser_names = metainfo
            .grammars
            .iter()
            .map(|grammar| grammar.name.to_string())
            .collect::<Vec<String>>();
          trace!("Rsvim.syn.loadParserSync result:{:?}", parser_names);
          let parser_names =
            parser_names.to_v8(scope, |scope, name| name.to_v8(scope).into());
          rv.set(parser_names.into());
        }
        Err(e) => {
          binding::throw_exception(scope, &e);
        }
      }
    }
    Err(_e) => {
      let e = TheErr::TreeSitterParserNotFound(options.grammar_path.clone());
      binding::throw_exception(scope, &e);
    }
  }
}
