use super::opt::*;

#[test]
fn test1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let opt1 = CommandOptionsBuilder::default().build().unwrap();
}
