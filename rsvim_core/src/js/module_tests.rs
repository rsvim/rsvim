use super::module::*;

use crate::js::JsRuntime;
use crate::test::js::make_js_runtime;
use crate::test::log::init as test_log_init;

#[test]
fn test_fetch1() {
  test_log_init();
  let mut jsrt = make_js_runtime();
  let scope = jsrt.handle_scope();
  // let module = fetch_module(&mut scope, filename, source);
}
