use super::converter::*;
use crate::cli::CliOptions;
use crate::tests::evloop::*;

#[test]
fn test_integer1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);
}
