use super::fs_loader::*;

use crate::js::loader::ModuleLoader;

use assert_fs::prelude::*;
use path_absolutize::Absolutize;
use std::fs;
use std::path::Path;

#[test]
fn test_resolve_fs_imports() {
  // Tests to run later on.
  let tests = vec![
    (
      None,
      "/dev/core/tests/005_more_imports.js",
      "/dev/core/tests/005_more_imports.js",
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
  ];

  // Run tests.
  let loader = FsModuleLoader {};

  for (base, specifier, expected) in tests {
    let path = loader.resolve(base, specifier).unwrap();
    let expected = if cfg!(target_os = "windows") {
      String::from(Path::new(expected).absolutize().unwrap().to_str().unwrap())
    } else {
      expected.into()
    };

    assert_eq!(path, expected);
  }
}

#[test]
fn test_load_fs_imports() {
  // Crate temp dir.
  let temp_dir = assert_fs::TempDir::new().unwrap();

  const SRC: &str = r"
            export function sayHello() {
                console.log('Hello, World!');
            }
        ";

  let source_files = [
    "./core/tests/005_more_imports.js",
    "./core/tests/006_more_imports/index.js",
  ];

  // Create source files.
  source_files.iter().for_each(|file| {
    let path = Path::new(file);
    let path = temp_dir.child(path);

    path.touch().unwrap();
    fs::write(path, SRC).unwrap();
  });

  // Group of tests to be run.
  let tests = vec![
    "./core/tests/005_more_imports",
    "./core/tests/005_more_imports.js",
    "./core/tests/006_more_imports/",
  ];

  // Run tests.
  let loader = FsModuleLoader {};

  for specifier in tests {
    let path = format!("{}", temp_dir.child(specifier).display());
    let source = loader.load(&path);

    assert!(source.is_ok());
    assert_eq!(source.unwrap(), SRC);
  }
}

// #[test]
// fn test_resolve_url_imports() {
//   // Group of tests to be run.
//   let tests = vec![
//     (
//       None,
//       "http://github.com/x/core/tests/006_url_imports.js",
//       "http://github.com/x/core/tests/006_url_imports.js",
//     ),
//     (
//       Some("http://github.com/x/core/tests/006_url_imports.js"),
//       "./005_more_imports.js",
//       "http://github.com/x/core/tests/005_more_imports.js",
//     ),
//     (
//       Some("http://github.com/x/core/tests/006_url_imports.js"),
//       "../005_more_imports.js",
//       "http://github.com/x/core/005_more_imports.js",
//     ),
//     (
//       Some("http://github.com/x/core/tests/006_url_imports.js"),
//       "http://github.com/x/core/tests/005_more_imports.js",
//       "http://github.com/x/core/tests/005_more_imports.js",
//     ),
//   ];
//
//   // Run tests.
//   let loader = UrlModuleLoader::default();
//
//   for (base, specifier, expected) in tests {
//     let url = loader.resolve(base, specifier).unwrap();
//     assert_eq!(url, expected);
//   }
// }
