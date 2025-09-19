use super::fs_loader::*;
use crate::js::loader::AsyncModuleLoader;
use crate::js::loader::ModuleLoader;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use assert_fs::prelude::*;
use normpath::PathExt;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn file_path1() {
  test_log_init();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - 005_more_imports.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("005_more_imports.js"), src),
  ]);

  let base: Option<&str> = None;
  let specifier = path_cfg.config_home().join("005_more_imports.js");
  let specifier = specifier.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual =
    loader.resolve(&path_cfg.config_home().to_string_lossy(), &specifier);
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

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/005_more_imports.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/tests/006_more_imports.js"), src),
  ]);

  let base = path_cfg.config_home().join("core").join("tests");
  let base = base.to_string_lossy().to_string();
  let specifier = "./006_more_imports.js";
  let expect = path_cfg
    .config_home()
    .join("core")
    .join("tests")
    .join("006_more_imports.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, specifier);
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

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/006_more_imports.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/006_more_imports.js"), src),
  ]);

  let base = path_cfg.config_home().join("core/tests/");
  let base = base.to_string_lossy().to_string();
  let specifier = "../006_more_imports.js";
  let expect = path_cfg.config_home().join("core/006_more_imports.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, specifier);
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

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/tests/006_more_imports.js"), src),
  ]);

  let base = path_cfg.config_home().to_string_lossy().to_string();
  let specifier = path_cfg
    .config_home()
    .join("core/tests/006_more_imports.js");
  let specifier = specifier.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, &specifier);
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

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/tests/006_more_imports.js"), ""),
  ]);

  let base = path_cfg.config_home().join("../core/tests/");
  let base = base.to_string_lossy().to_string();
  let specifier = path_cfg
    .config_home()
    .join("../core/tests/006_more_imports.js");
  let specifier = specifier.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(&base, &specifier);
  assert!(actual.is_err());
  info!("resolve error: {:?}", actual);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn file_path5() {
  test_log_init();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - 006_more_imports.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("006_more_imports.js"), src),
  ]);

  let specifier = "./006_more_imports.js";
  let expect = path_cfg.config_home().join("006_more_imports.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual =
    loader.resolve(&path_cfg.config_home().to_string_lossy(), specifier);
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

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - 006_more_imports.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("006_more_imports.js"), src),
  ]);

  let specifier = "006_more_imports.js";

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual =
    loader.resolve(&path_cfg.config_home().to_string_lossy(), specifier);
  assert!(actual.is_err());
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn folder_path1() {
  test_log_init();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/tests/006_more_imports/index.js"), src),
  ]);

  let base = path_cfg.config_home().join("core/tests/");
  let base = base.to_string_lossy().to_string();
  let specifier = "./006_more_imports";
  let expect = path_cfg
    .config_home()
    .join("core/tests/006_more_imports/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, specifier);
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

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/006_more_imports/index.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/006_more_imports/index.js"), src),
  ]);

  let base = path_cfg.config_home().join("core/tests");
  let base = base.to_string_lossy().to_string();
  let specifier = "../006_more_imports/";
  let expect = path_cfg
    .config_home()
    .join("core/006_more_imports/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, specifier);
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

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/006_more_imports/index.js"), src),
  ]);

  let specifier = path_cfg.config_home().join("core/006_more_imports/");
  let specifier = specifier.to_string_lossy().to_string();
  let expect = path_cfg
    .config_home()
    .join("core/006_more_imports/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual =
    loader.resolve(&path_cfg.config_home().to_string_lossy(), &specifier);
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

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/tests/006_more_imports/index.js"), src),
  ]);

  let base = path_cfg.config_home().join("core/tests");
  let base = base.to_string_lossy().to_string();
  let specifier = path_cfg.config_home().join("core/tests/006_more_imports/");
  let specifier = specifier.to_string_lossy().to_string();
  let expect = path_cfg
    .config_home()
    .join("core/tests/006_more_imports/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, &specifier);
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

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - core/tests/006_more_imports/index.js
  let (_tp, path_cfg) = make_configs(vec![
    (Path::new("rsvim.js"), ""),
    (Path::new("core/006_more_imports/index.js"), src),
  ]);

  let base = path_cfg.config_home().join("core/tests");
  let base = base.to_string_lossy().to_string();
  let specifier = "./006_more_imports";

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(&base, specifier);
  assert!(actual.is_err());
}

#[test]
fn folder_path_failed6() {
  test_log_init();
  let tp = TempPathConfig::create();

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

  let base = tp.xdg_config_home.child("rsvim/core/tests");
  let base = base.to_string_lossy().to_string();
  let specifier = tp
    .xdg_config_home
    .child("rsvim/core/tests/006_more_imports");
  let specifier = specifier.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(&base, &specifier);
  assert!(actual.is_err());
}

#[test]
fn folder_path_failed7() {
  test_log_init();
  let tp = TempPathConfig::create();

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

  let specifier = tp.xdg_config_home.child("rsvim/core/006_more_imports/");
  let specifier = specifier.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(
    &tp.xdg_config_home.join("rsvim").to_string_lossy(),
    &specifier,
  );
  assert!(actual.is_err());
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn npm_package1() {
  test_log_init();
  let tp = TempPathConfig::create();

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

  let base = tp.xdg_config_home.child("rsvim/core/tests/");
  let base = base.to_string_lossy().to_string();
  let specifier = "./006_more_imports";
  let expect = tp
    .xdg_config_home
    .child("rsvim/core/tests/006_more_imports/lib/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, specifier);
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
  let tp = TempPathConfig::create();

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

  let base = tp.xdg_config_home.child("rsvim");
  let base = base.to_string_lossy().to_string();
  let specifier = "./006_more_imports/";
  let expect = tp
    .xdg_config_home
    .child("rsvim/node_modules/006_more_imports/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, specifier);
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
async fn npm_package3() {
  test_log_init();
  let tp = TempPathConfig::create();

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
  let expect = tp
    .xdg_config_home
    .child("rsvim/node_modules/006_more_imports/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(
    &tp.xdg_config_home.join("rsvim").to_string_lossy(),
    specifier,
  );
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
async fn npm_package4() {
  test_log_init();
  let tp = TempPathConfig::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg: &str = r#"
{
  "exports": "./index.js"
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - node_modules/006_more_imports/index.js
  // - node_modules/006_more_imports/package.json
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("node_modules/006_more_imports/index.js"), src),
      (Path::new("node_modules/006_more_imports/package.json"), pkg),
    ],
  );

  let specifier = "006_more_imports";
  let expect = tp
    .xdg_config_home
    .child("rsvim/node_modules/006_more_imports/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(
    &tp.xdg_config_home.join("rsvim").to_string_lossy(),
    specifier,
  );
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
async fn npm_package5() {
  test_log_init();
  let tp = TempPathConfig::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg: &str = r#"
{
  "exports": "./dist/index.js"
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - node_modules/006_more_imports/dist/index.js
  // - node_modules/006_more_imports/package.json
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (
        Path::new("node_modules/006_more_imports/dist/index.js"),
        src,
      ),
      (Path::new("node_modules/006_more_imports/package.json"), pkg),
    ],
  );

  let base = tp.xdg_config_home.child("rsvim/");
  let base = base.to_string_lossy().to_string();
  let specifier = "006_more_imports";
  let expect = tp
    .xdg_config_home
    .child("rsvim/node_modules/006_more_imports/dist/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, specifier);
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
async fn npm_package6() {
  test_log_init();
  let tp = TempPathConfig::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg: &str = r#"
{
  "exports": "./dist/index.js"
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - node_modules/006_more_imports/dist/index.js
  // - node_modules/006_more_imports/package.json
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (
        Path::new("node_modules/006_more_imports/dist/index.js"),
        src,
      ),
      (Path::new("node_modules/006_more_imports/package.json"), pkg),
    ],
  );

  let base = tp.xdg_config_home.child("rsvim/node_modules");
  let base = base.to_string_lossy().to_string();
  let specifier = "006_more_imports";
  let expect = tp
    .xdg_config_home
    .child("rsvim/node_modules/006_more_imports/dist/index.js");
  let expect = expect.to_string_lossy().to_string();

  // Run tests.
  let loader = FsModuleLoader::new();
  let aloader = AsyncFsModuleLoader {};

  let actual = loader.resolve(&base, specifier);
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
async fn npm_package_failed7() {
  test_log_init();
  let tp = TempPathConfig::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg: &str = r#"
{
  "exports": "./dist/index.js"
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - node_modules/006_more_imports/index.js
  // - node_modules/006_more_imports/package.json
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("node_modules/006_more_imports/index.js"), src),
      (Path::new("node_modules/006_more_imports/package.json"), pkg),
    ],
  );

  let specifier = "006_more_imports";

  // Run tests.
  let loader = FsModuleLoader::new();

  let actual = loader.resolve(
    &tp.xdg_config_home.join("rsvim").to_string_lossy(),
    specifier,
  );
  assert!(actual.is_err());
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn node_builtin_module1() {
  test_log_init();
  let tp = TempPathConfig::create();

  let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

  let pkg: &str = r#"
{
  "main": "./index.js"
}
"#;

  // Prepare $RSVIM_CONFIG:
  // - rsvim.js
  // - node_modules/os/index.js
  // - node_modules/os/package.json
  make_configs(
    &tp,
    vec![
      (Path::new("rsvim.js"), ""),
      (Path::new("node_modules/os/index.js"), src),
      (Path::new("node_modules/os/package.json"), pkg),
      (Path::new("node_modules/fs/index.js"), src),
      (Path::new("node_modules/fs/package.json"), pkg),
      (Path::new("node_modules/net/index.js"), src),
      (Path::new("node_modules/net/package.json"), pkg),
      (Path::new("node_modules/path/index.js"), src),
      (Path::new("node_modules/path/package.json"), pkg),
    ],
  );

  // os
  {
    let specifier = "os";
    let expect = tp.xdg_config_home.child("rsvim/node_modules/os/index.js");
    let expect = expect.to_string_lossy().to_string();

    // Run tests.
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

    let actual = loader.resolve(
      &tp.xdg_config_home.join("rsvim").to_string_lossy(),
      specifier,
    );
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

  // fs
  {
    let specifier = "fs";
    let expect = tp.xdg_config_home.child("rsvim/node_modules/fs/index.js");
    let expect = expect.to_string_lossy().to_string();

    // Run tests.
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

    let actual = loader.resolve(
      &tp.xdg_config_home.join("rsvim").to_string_lossy(),
      specifier,
    );
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

  // net
  {
    let specifier = "net";
    let expect = tp.xdg_config_home.child("rsvim/node_modules/net/index.js");
    let expect = expect.to_string_lossy().to_string();

    // Run tests.
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

    let actual = loader.resolve(
      &tp.xdg_config_home.join("rsvim").to_string_lossy(),
      specifier,
    );
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

  // path
  {
    let specifier = "path";
    let expect = tp.xdg_config_home.child("rsvim/node_modules/path/index.js");
    let expect = expect.to_string_lossy().to_string();

    // Run tests.
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

    let actual = loader.resolve(
      &tp.xdg_config_home.join("rsvim").to_string_lossy(),
      specifier,
    );
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
}
