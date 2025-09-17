use super::fs_loader::*;
use crate::js::loader::AsyncModuleLoader;
use crate::js::loader::ModuleLoader;
use crate::prelude::*;
use crate::tests::constant::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use assert_fs::prelude::*;
use normpath::PathExt;
use std::fs;
use std::path::Path;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn file_path1() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - 005_more_imports.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("005_more_imports.js"), src),
    ],
  );

  let base: Option<&str> = None;
  let specifier = transform(
    tp.xdg_config_home
      .child("rsvim")
      .child("005_more_imports.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(base, &specifier);
  info!(
    "base:{:?},specifier:{:?},actual:{:?}",
    base, specifier, actual,
  );
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&specifier).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn file_path2() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/005_more_imports.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/tests/006_more_imports.js"), src),
    ],
  );

  let base = transform(
    tp.xdg_config_home
      .child("rsvim")
      .child("core")
      .child("tests")
      .to_path_buf(),
  );
  let specifier = "./006_more_imports.js";
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim")
      .child("core")
      .child("tests")
      .child("006_more_imports.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(Some(&base), specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{:?},specifier:{:?},actual:{:?},expect:{:?}",
    base, specifier, actual, expect
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn file_path3() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/006_more_imports.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/006_more_imports.js"), src),
    ],
  );

  let base =
    transform(tp.xdg_config_home.child("rsvim/core/tests/").to_path_buf());
  let specifier = "../006_more_imports.js";
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim/core/006_more_imports.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(Some(&base), specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{:?},specifier:{:?},actual:{:?},expect:{:?}",
    base, specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn file_path4() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/tests/006_more_imports.js"), src),
    ],
  );

  let base = transform(tp.xdg_config_home.child("rsvim/").to_path_buf());
  let specifier = transform(
    tp.xdg_config_home
      .child("rsvim/core/tests/006_more_imports.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(Some(&base), &specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{:?},specifier:{:?},actual:{:?}",
    base, specifier, actual,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&specifier).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[test]
fn file_path_failed4() {
  test_log_init();
  let tp = TempPathCfg::create();

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/tests/006_more_imports.js"), ""),
    ],
  );

  let base = transform(tp.xdg_config_home.child("core/tests/").to_path_buf());
  let specifier = transform(
    tp.xdg_config_home
      .child("core/tests/006_more_imports.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(Some(&base), &specifier);
  assert!(actual.is_err());
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn file_path5() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - 006_more_imports.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("006_more_imports.js"), src),
    ],
  );

  let specifier = "./006_more_imports.js";
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim")
      .child("006_more_imports.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(None, specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:None,specifier:{:?},actual:{:?},expect:{:?}",
    specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[test]
fn file_path_failed5() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - 006_more_imports.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("006_more_imports.js"), src),
    ],
  );

  let specifier = "006_more_imports.js";

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(None, specifier);
  assert!(actual.is_err());
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn folder_path1() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/tests/006_more_imports/index.js"), src),
    ],
  );

  let base =
    transform(tp.xdg_config_home.child("rsvim/core/tests/").to_path_buf());
  let specifier = "./006_more_imports";
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim/core/tests/006_more_imports/index.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(Some(&base), specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{base:?},specifier:{:?},actual:{:?},expect:{:?}",
    specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn folder_path2() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/006_more_imports/index.js"), src),
    ],
  );

  let base =
    transform(tp.xdg_config_home.child("rsvim/core/tests").to_path_buf());
  let specifier = "../006_more_imports/";
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim/core/006_more_imports/index.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(Some(&base), specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{:?},specifier:{:?},actual:{:?},expect:{:?}",
    base, specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn folder_path3() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/006_more_imports/index.js"), src),
    ],
  );

  let specifier = transform(
    tp.xdg_config_home
      .child("rsvim/core/006_more_imports/")
      .to_path_buf(),
  );
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim/core/006_more_imports/index.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(None, &specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "specifier:{:?},actual:{:?},expect:{:?}",
    specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn folder_path4() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/tests/006_more_imports/index.js"), src),
    ],
  );

  let base =
    transform(tp.xdg_config_home.child("rsvim/core/tests").to_path_buf());
  let specifier = transform(
    tp.xdg_config_home
      .child("rsvim/core/tests/006_more_imports/")
      .to_path_buf(),
  );
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim/core/tests/006_more_imports/index.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(Some(&base), &specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{:?},specifier:{:?},actual:{:?},expect:{:?}",
    base, specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[test]
fn folder_path_failed5() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/006_more_imports/index.js"), src),
    ],
  );

  let base =
    transform(tp.xdg_config_home.child("rsvim/core/tests").to_path_buf());
  let specifier = "./006_more_imports";

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(Some(&base), specifier);
  assert!(actual.is_err());
}

#[test]
fn folder_path_failed6() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/006_more_imports/index.js"), src),
    ],
  );

  let base =
    transform(tp.xdg_config_home.child("rsvim/core/tests").to_path_buf());
  let specifier = transform(
    tp.xdg_config_home
      .child("rsvim/core/tests/006_more_imports")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(Some(&base), &specifier);
  assert!(actual.is_err());
}

#[test]
fn folder_path_failed7() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/006_more_imports/lib.js"), src),
    ],
  );

  let specifier = transform(
    tp.xdg_config_home
      .child("rsvim/core/006_more_imports/")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(None, &specifier);
  assert!(actual.is_err());
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn npm_package1() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg_src: &str = r#"
{
  "main": "./lib/index.js"
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/lib/index.js
  // - core/tests/006_more_imports/package.json
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("core/tests/006_more_imports/lib/index.js"), src),
      (
        Path::new("core/tests/006_more_imports/package.json"),
        pkg_src,
      ),
    ],
  );

  let base =
    transform(tp.xdg_config_home.child("rsvim/core/tests/").to_path_buf());
  let specifier = "./006_more_imports";
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim/core/tests/006_more_imports/lib/index.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(Some(&base), specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{:?},specifier:{:?},actual:{:?},expect:{:?}",
    base, specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn npm_package2() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - node_modules/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("node_modules/006_more_imports/index.js"), src),
    ],
  );

  let base = transform(tp.xdg_config_home.child("rsvim").to_path_buf());
  let specifier = "./006_more_imports/";
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim/node_modules/006_more_imports/index.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(Some(&base), specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{:?},specifier:{:?},actual:{:?},expect:{:?}",
    base, specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn npm_package5() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - node_modules/006_more_imports/index.js
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("node_modules/006_more_imports/index.js"), src),
    ],
  );

  let specifier = "006_more_imports";
  let expect = transform(
    tp.xdg_config_home
      .child("rsvim/node_modules/006_more_imports/index.js")
      .to_path_buf(),
  );

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(None, specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:{:?},specifier:{:?},actual:{:?},expect:{:?}",
    base, specifier, actual, expect,
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module1 = loader.load(&actual);
  assert!(actual_module1.is_ok());
  assert_eq!(actual_module1.unwrap(), src);

  let actual_module2 = aloader.load(&actual).await;
  assert!(actual_module2.is_ok());
  assert_eq!(actual_module2.unwrap(), src);
}

#[test]
fn npm_package3() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg_src: &str = r#"
{
  "exports": "./dist/index.js"
}
"#;

  let entry = tp.xdg_config_home.child("rsvim").child("rsvim.js");
  let pkg = tp
    .xdg_config_home
    .child("rsvim")
    .child("node_modules")
    .child("006_more_imports")
    .child("package.json");
  let specifier = "006_more_imports/";
  let expect = tp
    .xdg_config_home
    .child("rsvim")
    .child("node_modules")
    .child("006_more_imports")
    .child("dist")
    .child("index.js");

  // Run tests.
  let loader = FsModuleLoader::new();

  // Prepare configs
  {
    entry.touch().unwrap();
    expect.touch().unwrap();
    fs::write(expect.path(), src).unwrap();
    pkg.touch().unwrap();
    fs::write(pkg.path(), pkg_src).unwrap();
  }

  let expect = transform(expect.to_path_buf());

  let actual = loader.resolve(None, specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:None,specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
    specifier,
    actual,
    expect,
    expect.replace("/", "\\")
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module = loader.load(&actual);
  assert!(actual_module.is_ok());
  assert_eq!(actual_module.unwrap(), src);
}

#[test]
fn npm_package4() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg_src: &str = r#"
{
  "exports": "./dist/index.js"
}
"#;

  let entry = tp.xdg_config_home.child("rsvim").child("rsvim.js");
  let pkg = tp
    .xdg_config_home
    .child("rsvim")
    .child("006_more_imports")
    .child("package.json");
  let specifier = "006_more_imports";
  let expect = tp
    .xdg_config_home
    .child("rsvim")
    .child("006_more_imports")
    .child("dist")
    .child("index.js");

  // Run tests.
  let loader = FsModuleLoader::new();

  // Prepare configs
  {
    entry.touch().unwrap();
    expect.touch().unwrap();
    fs::write(expect.path(), src).unwrap();
    pkg.touch().unwrap();
    fs::write(pkg.path(), pkg_src).unwrap();
  }

  let expect = transform(expect.to_path_buf());

  let actual = loader.resolve(None, specifier);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!(
    "base:None,specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
    specifier,
    actual,
    expect,
    expect.replace("/", "\\")
  );
  assert_eq!(
    Path::new(&actual).normalize().unwrap(),
    Path::new(&expect).normalize().unwrap()
  );

  let actual_module = loader.load(&actual);
  assert!(actual_module.is_ok());
  assert_eq!(actual_module.unwrap(), src);
}

#[test]
fn npm_package_failed5() {
  test_log_init();
  let tp = TempPathCfg::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg_src: &str = r#"
{
  "exports": "./dist/index.js"
}
"#;

  let entry = tp.xdg_config_home.child("rsvim").child("rsvim.js");
  let pkg = tp
    .xdg_config_home
    .child("rsvim")
    .child("006_more_imports")
    .child("package.json");
  let specifier = "006_more_imports";
  let expect = tp
    .xdg_config_home
    .child("rsvim")
    .child("006_more_imports")
    .child("lib")
    .child("index.js");

  // Run tests.
  let loader = FsModuleLoader::new();

  // Prepare configs
  {
    entry.touch().unwrap();
    expect.touch().unwrap();
    fs::write(expect.path(), src).unwrap();
    pkg.touch().unwrap();
    fs::write(pkg.path(), pkg_src).unwrap();
  }

  let actual = loader.resolve(None, specifier);
  assert!(actual.is_err());
}
