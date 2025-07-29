use super::module::*;

use crate::js::JsRuntime;
use crate::prelude::*;
use crate::test::constant::acquire_sequential_guard;
use crate::test::js::make_js_runtime;
use crate::test::log::init as test_log_init;

use assert_fs::TempDir;
use std::io::Write;

#[test]
fn fetch1() {
  let _guard = acquire_sequential_guard();
  test_log_init();

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
  test_log_init();

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

  let mut jsrt = make_js_runtime();
  let mut scope = jsrt.handle_scope();
  let actual2 =
    fetch_module(&mut scope, fetch2.as_os_str().to_str().unwrap(), None);
  assert!(actual2.is_none());
}

#[test]
fn fetch_tree3() {
  let _guard = acquire_sequential_guard();
  test_log_init();

  let tmpdir = TempDir::new().unwrap();

  let src1: &str = r#"
  export const PI = 3.14159;
  "#;

  let src2: &str = r#"
  import { PI } from "./pi.js";

  function addPI(a:number, b:number) :number {
    return PI+a+b;
  }

  export { addPI };
  "#;

  let src3: &str = r#"
  import * as pi from "./util/pi.js";
  import addUtil from "./util/add";

  const result = addUtil.addPI(1.0, 2.5);
  "#;

  let tmp_util_dir = tmpdir.join("util");
  let fetch1 = tmp_util_dir.join("pi.js");
  let fetch2 = tmp_util_dir.join("add.ts");
  let fetch3 = tmpdir.join("fetch3.js");

  {
    std::fs::create_dir_all(tmp_util_dir).unwrap();

    let mut fp1 = std::fs::File::create(&fetch1).unwrap();
    fp1.write_all(src1.as_bytes()).unwrap();
    fp1.flush().unwrap();

    let mut fp2 = std::fs::File::create(&fetch2).unwrap();
    fp2.write_all(src2.as_bytes()).unwrap();
    fp2.flush().unwrap();

    let mut fp3 = std::fs::File::create(&fetch3).unwrap();
    fp3.write_all(src3.as_bytes()).unwrap();
    fp3.flush().unwrap();
  }

  let mut jsrt = make_js_runtime();
  let mut scope = jsrt.handle_scope();
  let actual1 =
    fetch_module_tree(&mut scope, fetch3.as_os_str().to_str().unwrap(), None);
  assert!(actual1.is_some());
  let actual1 = actual1.unwrap();
  info!(
    "fetch_tree3 actual1:{:?}, script_id:{:?}",
    actual1,
    actual1.script_id()
  );
  assert!(actual1.script_id().is_some());
  assert!(actual1.script_id().unwrap() > 0);

  let state = JsRuntime::state(&scope);
  let state = state.borrow();

  let path3 = resolve_import(None, fetch3.to_str().unwrap(), None);
  assert!(path3.is_ok());
  let path3 = path3.unwrap();
  assert!(state.module_map.seen().borrow().contains_key(&path3));

  let path1 = resolve_import(
    Some(fetch3.to_str().unwrap()),
    fetch1.to_str().unwrap(),
    None,
  );
  assert!(path1.is_ok());
  let path1 = path1.unwrap();
  assert!(state.module_map.seen().borrow().contains_key(&path1));

  let path2 = resolve_import(
    Some(fetch3.to_str().unwrap()),
    fetch2.to_str().unwrap(),
    None,
  );
  assert!(path2.is_ok());
  let path2 = path2.unwrap();
  assert!(state.module_map.seen().borrow().contains_key(&path2));
}
