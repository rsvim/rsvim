use super::fs_loader::*;

use crate::constant::PathConfig;
use crate::js::loader::ModuleLoader;
use crate::test::log::init as test_log_init;

use assert_fs::prelude::*;
use std::fs;
use std::path::Path;
use tracing::info;

#[test]
fn test_resolve1() {
  test_log_init();

  // Tests to run later on.
  let tests = vec![
    (
      None,
      "/dev/core/tests/005_more_imports.js",
      "005_more_imports.js",
    ),
    (
      Some("/dev/core/tests/005_more_imports.js"),
      "./006_more_imports.js",
      "/dev/core/tests/006_more_imports.js",
    ),
    (
      Some("/dev/core/tests/005_more_imports.js"),
      "../006_more_imports.js",
      "/dev/core/006_more_imports.js",
    ),
    (
      Some("/dev/core/tests/005_more_imports.js"),
      "/dev/core/tests/006_more_imports.js",
      "/dev/core/tests/006_more_imports.js",
    ),
    (
      Some("/dev/core/tests/005_more_imports.js"),
      "./006_more_imports",
      "/dev/core/tests/006_more_imports",
    ),
    (
      Some("/dev/core/tests/005_more_imports.js"),
      "./006_more_imports/",
      "/dev/core/tests/006_more_imports",
    ),
  ];

  // Run tests.
  let loader = FsModuleLoader {};

  let path_cfg = PathConfig::new();
  for (base, specifier, expect) in tests {
    let actual = loader.resolve(&path_cfg, base, specifier).unwrap();
    info!(
      "base:{base:?},specifier:{specifier:?},actual:{actual:?},expect:{expect:?},expect(\\):{:?}",
      expect.replace("/", "\\")
    );
    if cfg!(target_os = "windows") {
      assert!(actual == expect || actual.ends_with(&expect.replace("/", "\\")));
    } else {
      assert!(actual == expect || actual.ends_with(expect));
    }
  }
}

#[test]
fn test_load1() {
  test_log_init();
  // Crate temp dir.
  let temp_dir = assert_fs::TempDir::new().unwrap();

  let src: &str = r#"
      export function sayHello() {
          console.log('Hello, World!');
      }
  "#;

  let source_files = [
    "./core/tests/005_more_imports.js",
    "./core/tests/006_more_imports/index.js",
  ];

  // Create source files.
  source_files.iter().for_each(|file| {
    let path = Path::new(file);
    let path = temp_dir.child(path);

    path.touch().unwrap();
    fs::write(path, src).unwrap();
  });

  // Group of tests to be run.
  let tests = vec![
    "./core/tests/005_more_imports",
    "./core/tests/005_more_imports.js",
    "./core/tests/006_more_imports/",
    "./core/tests/006_more_imports",
  ];

  // Run tests.
  let loader = FsModuleLoader {};

  for specifier in tests {
    let path = format!("{}", temp_dir.child(specifier).display());
    let source = loader.load(&path);
    info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
    assert!(source.is_ok());
    assert_eq!(source.unwrap(), src);
  }
}

#[test]
fn test_load2() {
  test_log_init();
  // Crate temp dir.
  let temp_dir = assert_fs::TempDir::new().unwrap();

  let src: &str = r#"
  {
    "name": 1
  }
  "#;

  let source_files = [
    "./core/tests/005_more_imports.json",
    "./core/tests/006_more_imports/index.json",
  ];

  // Create source files.
  source_files.iter().for_each(|file| {
    let path = Path::new(file);
    let path = temp_dir.child(path);

    path.touch().unwrap();
    fs::write(path, src).unwrap();
  });

  // Group of tests to be run.
  let tests = vec![
    "./core/tests/005_more_imports",
    "./core/tests/005_more_imports.json",
    "./core/tests/006_more_imports/",
    "./core/tests/006_more_imports",
    "./core/tests/006_more_imports/index.json",
  ];

  // Run tests.
  let loader = FsModuleLoader {};

  for specifier in tests {
    let path = format!("{}", temp_dir.child(specifier).display());
    let source = loader.load(&path);
    info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
    assert!(source.is_ok());
    assert!(source.unwrap().contains(src));
  }
}
