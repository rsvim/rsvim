//! Load tree-sitter grammar APIs.

use crate::from_v8_prop;
use crate::js::JsFuture;
use crate::js::binding;
use crate::js::converter::*;
use crate::prelude::*;
use crate::to_v8_prop;
use compact_str::CompactString;
use compact_str::ToCompactString;

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

pub struct SynLoadTreeSitterGrammarFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for SynLoadTreeSitterGrammarFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|SynLoadTreeSitterGrammarFuture|");

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
