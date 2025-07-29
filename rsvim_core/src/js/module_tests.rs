use super::module::*;

use crate::prelude::*;
use crate::test::constant::acquire_sequential_guard;
use crate::test::js::make_js_runtime;
use crate::test::log::init as test_log_init;

use assert_fs::TempDir;
use std::io::Write;

#[test]
fn fetch1() {
  let _guard = acquire_sequential_guard();

  let tmpdir = TempDir::new().unwrap();

  let src1: &str = r#"
  export const PI = 3.14159;
  export function Hello(a, b) {
    return a+b;
  }
  function World(a, b) {
    return a-b;
  }
  export { World };
  "#;

  let fetch1 = tmpdir.join("fetch1.js");

  {
    let mut fp = std::fs::File::create(&fetch1).unwrap();
    fp.write_all(src1.as_bytes()).unwrap();
    fp.flush().unwrap();
  }

  test_log_init();
  let mut jsrt = make_js_runtime();
  let mut scope = jsrt.handle_scope();
  let actual1 =
    fetch_module(&mut scope, fetch1.as_os_str().to_str().unwrap(), None);
  assert!(actual1.is_some());
  let actual1 = actual1.unwrap();
  info!(
    "fetch1 actual1:{:?}, script_id:{:?}",
    actual1,
    actual1.script_id()
  );
  assert!(actual1.script_id().is_some());
  assert!(actual1.script_id().unwrap() > 0);
}

#[test]
fn fetch2() {
  let _guard = acquire_sequential_guard();

  let tmpdir = TempDir::new().unwrap();

  // Actually it's rust code...
  let src2: &str = r#"
  #[test]
  fn fetch2() {
    println!("hello");
  }
  "#;

  let fetch2 = tmpdir.join("fetch2.js");

  {
    let mut fp = std::fs::File::create(&fetch2).unwrap();
    fp.write_all(src2.as_bytes()).unwrap();
    fp.flush().unwrap();
  }

  test_log_init();
  let mut jsrt = make_js_runtime();
  let mut scope = jsrt.handle_scope();
  let actual2 =
    fetch_module(&mut scope, fetch2.as_os_str().to_str().unwrap(), None);
  assert!(actual2.is_none());
}
