use super::module::*;
use crate::js::JsRuntime;
use crate::prelude::*;
use crate::tests::js::make_js_runtime;
use crate::tests::log::init as test_log_init;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use std::io::Write;

#[test]
#[cfg_attr(miri, ignore)]
fn fetch1() {
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
#[cfg_attr(miri, ignore)]
fn fetch2() {
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
#[cfg_attr(miri, ignore)]
fn fetch_tree3() {
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
  import addUtil from "./util/add.ts";

  const result = addUtil.addPI(1.0, 2.5);
  "#;

  let tmp_util_dir = tmpdir.child("util");
  let fetch1 = tmp_util_dir.child("pi.js");
  let fetch2 = tmp_util_dir.child("add.ts");
  let fetch3 = tmpdir.child("fetch3.js");

  {
    fetch1.touch().unwrap();
    std::fs::write(fetch1.path(), src1).unwrap();

    fetch2.touch().unwrap();
    std::fs::write(fetch2.path(), src2).unwrap();

    fetch3.touch().unwrap();
    std::fs::write(fetch3.path(), src3).unwrap();
  }

  let mut jsrt = make_js_runtime();
  let mut scope = jsrt.handle_scope();
  let actual1 = fetch_module_tree(
    &mut scope,
    fetch3.path().as_os_str().to_str().unwrap(),
    None,
  );
  assert!(actual1.is_some());
  let actual1 = actual1.unwrap();
  info!(
    "fetch_tree3 actual1:{:?}, script_id:{:?}",
    actual1,
    actual1.script_id()
  );
  assert!(actual1.script_id().is_some());
  assert!(actual1.script_id().unwrap() > 0);

  let state_rc = JsRuntime::state(&scope);
  let state = state_rc.borrow();

  let path3 = resolve_import(None, fetch3.path().to_str().unwrap(), None);
  info!("fetch_tree3 path3:{:?}, fetch3:{:?}", path3, fetch3.path());
  assert!(path3.is_ok());
  let path3 = path3.unwrap();
  assert!(state.module_map.get(&path3).is_some());

  let path1 = resolve_import(
    Some(tmpdir.path().to_str().unwrap()),
    fetch1.to_str().unwrap(),
    None,
  );
  info!("fetch_tree3 path1:{:?}, fetch1:{:?}", path1, fetch1.path());
  assert!(path1.is_ok());
  let path1 = path1.unwrap();
  assert!(state.module_map.get(&path1).is_some());

  let fetch2_without_ext =
    fetch2.parent().unwrap().join(fetch2.file_stem().unwrap());
  info!(
    "fetch_tree3 fetch2:{:?},fetch2.file_stem:{:?},fetch2.without_extension:{:?}",
    fetch2.path(),
    fetch2.file_stem(),
    fetch2_without_ext
  );
  let path2 = resolve_import(
    Some(tmpdir.path().to_str().unwrap()),
    fetch2_without_ext.to_str().unwrap(),
    None,
  );
  info!("fetch_tree3 path2:{:?}, fetch2:{:?}", path2, fetch2.path());
  assert!(path2.is_err());
}

#[test]
#[cfg_attr(miri, ignore)]
fn fetch_tree4() {
  test_log_init();

  let tmpdir = TempDir::new().unwrap();

  let src1: &str = r#"
  export const PI = 3.14159;
  "#;

  let src2: &str = r#"
  import { PI } from "./index.js";

  function addPI(a:number, b:number) :number {
    return PI+a+b;
  }

  export { addPI };
  "#;

  let src3: &str = r#"
  import * as pi from "./util";
  import addUtil from "./util/add.ts";

  const result = addUtil.addPI(1.0, 2.5);
  "#;

  let tmp_util_dir = tmpdir.join("util");
  let fetch1 = tmp_util_dir.join("index.js");
  let fetch2 = tmp_util_dir.join("add.ts");
  let fetch3 = tmpdir.join("index.js");

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
  let actual1 = fetch_module_tree(&mut scope, fetch3.to_str().unwrap(), None);
  assert!(actual1.is_some());
  let actual1 = actual1.unwrap();
  info!(
    "fetch_tree4 actual1:{:?}, script_id:{:?}",
    actual1,
    actual1.script_id()
  );
  assert!(actual1.script_id().is_some());
  assert!(actual1.script_id().unwrap() > 0);

  let state = JsRuntime::state(&scope);
  let state = state.borrow();

  let fetch3_path = resolve_import(None, fetch3.to_str().unwrap(), None);
  info!(
    "fetch_tree4 fetch3_path:{:?}, fetch3:{:?}",
    fetch3_path, fetch3
  );
  assert!(fetch3_path.is_ok());
  let fetch3_path = fetch3_path.unwrap();
  assert!(state.module_map.get(&fetch3_path).is_some());

  let fetch1_path = resolve_import(
    Some(tmpdir.to_str().unwrap()),
    fetch1.to_str().unwrap(),
    None,
  );
  info!(
    "fetch_tree4 fetch1_path:{:?}, fetch1:{:?}",
    fetch1_path, fetch1
  );
  assert!(fetch1_path.is_ok());
  let fetch1_path = fetch1_path.unwrap();
  assert!(state.module_map.get(&fetch1_path).is_some());

  let fetch2_without_ext =
    fetch2.parent().unwrap().join(fetch2.file_stem().unwrap());
  info!(
    "fetch_tree4 fetch2:{:?},fetch2.file_stem:{:?},fetch2.without_extension:{:?}",
    fetch2,
    fetch2.file_stem(),
    fetch2_without_ext
  );
  let fetch2_path = resolve_import(
    Some(tmpdir.to_str().unwrap()),
    fetch2_without_ext.to_str().unwrap(),
    None,
  );
  info!(
    "fetch_tree4 fetch2_path:{:?}, fetch2:{:?}",
    fetch2_path, fetch2
  );
  assert!(fetch2_path.is_err());
}
