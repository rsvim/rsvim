use super::fs_loader::*;

use crate::js::loader::{AsyncModuleLoader, ModuleLoader};
use crate::prelude::*;
use crate::tests::constant::*;
use crate::tests::log::init as test_log_init;

use assert_fs::prelude::*;
use normpath::PathExt;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod tests_sync_resolve {
  use super::*;

  #[test]
  fn resolve_file1() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base: Option<&str> = None;
    let specifier = temp_dir.child("005_more_imports.js");
    let expect = temp_dir.child("005_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      specifier.touch().unwrap();
      fs::write(specifier.path(), src).unwrap();
    }
    let specifier = transform(specifier.to_path_buf());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, &specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_file2() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = "./006_more_imports.js";
    let expect = temp_dir.child("core/tests/006_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      base.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(actual, expect);
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_file3() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = "../006_more_imports.js";
    let expect = temp_dir.child("core/006_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      base.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(actual, expect);
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_file4() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = temp_dir.child("core/tests/006_more_imports.js");
    let expect = temp_dir.child("core/tests/006_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      base.touch().unwrap();
      specifier.touch().unwrap();
      fs::write(specifier.path(), src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let specifier = transform(specifier.to_path_buf());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, &specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(actual, expect);
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_folder1() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = "./006_more_imports";
    let expect = temp_dir.child("core/tests/006_more_imports/index.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      base.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(actual, expect);
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_folder2() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = "../006_more_imports/";
    let expect = temp_dir.child("core/006_more_imports/index.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      base.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(actual, expect);
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_folder3() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base: Option<&str> = None;
    let specifier = temp_dir.child("core/tests/006_more_imports/");
    let expect = temp_dir.child("core/tests/006_more_imports/index.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
    }

    let specifier = transform(specifier.to_path_buf());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, &specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(actual, expect);
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_folder4() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = temp_dir.child("core/tests/006_more_imports/");
    let expect = temp_dir.child("core/tests/006_more_imports/index.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      base.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let specifier = transform(specifier.to_path_buf());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, &specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(actual, expect);
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_npm_package1() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

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

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = "./006_more_imports";
    let pkg = temp_dir.child("core/tests/006_more_imports/package.json");
    let expect = temp_dir.child("core/tests/006_more_imports/lib/index.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      base.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
      pkg.touch().unwrap();
      fs::write(pkg.path(), pkg_src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_npm_package2() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let pkg_src: &str = r#"
{
  "exports": {
    ".": "./src/index.js"
  }
}
"#;

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = "../006_more_imports/";
    let pkg = temp_dir.child("core/006_more_imports/package.json");
    let expect = temp_dir.child("core/006_more_imports/src/index.js");

    // Run tests.
    let loader = FsModuleLoader {};

    // Prepare configs
    {
      base.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
      pkg.touch().unwrap();
      fs::write(pkg.path(), pkg_src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let expect = transform(expect.to_path_buf());

    let actual = loader.resolve(base, specifier);
    assert!(actual.is_ok());
    let actual = actual.unwrap();
    info!(
      "base:{base:?},specifier:{:?},actual:{:?},expect:{:?},expect(\\):{:?}",
      specifier,
      actual,
      expect,
      expect.replace("/", "\\")
    );
    // if cfg!(target_os = "windows") {
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_npm_package3() {
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
    let loader = FsModuleLoader {};

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
    // if cfg!(target_os = "windows") {
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }

  #[test]
  fn resolve_npm_package4() {
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
    let loader = FsModuleLoader {};

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
    // if cfg!(target_os = "windows") {
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );
    // } else {
    //   assert_eq!(actual, expect);
    // }
  }
}

#[cfg(test)]
mod tests_sync_load {
  use super::*;

  #[test]
  fn load_file1() {
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
      let path = loader.resolve(
        Some(
          temp_dir
            .path()
            .join("rsvim.js")
            .as_os_str()
            .to_str()
            .unwrap(),
        ),
        specifier,
      );
      assert!(path.is_ok());
      let path = path.unwrap();
      let source = loader.load(&path);
      info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
      assert!(source.is_ok());
      assert_eq!(source.unwrap(), src);
    }
  }

  #[test]
  fn load_files_dirs_not_found1() {
    test_log_init();
    // Crate temp dir.
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
      export function sayHello() {
          console.log('Hello, World!');
      }
  "#;

    let source_files = [
      "./core/tests/005_more_imports.cjs",
      "./core/tests/006_more_imports/index",
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
      assert!(source.is_err());
    }
  }

  #[test]
  fn load_json1() {
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

  #[test]
  fn load_node_module1() {
    test_log_init();
    // Crate temp dir.
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
      export function sayHello() {
          console.log('Hello, World!');
      }
  "#;
    let src_file: &str = "./core/tests/006_more_imports/index.js";

    let pkg: &str = r#"
      {
        "exports": "./index.js"
      }
    "#;
    let pkg_file: &str = "./core/tests/006_more_imports/package.json";

    // Create source files.
    [(src_file, src), (pkg_file, pkg)]
      .iter()
      .for_each(|(file, src)| {
        let path = Path::new(file);
        let path = temp_dir.child(path);

        path.touch().unwrap();
        fs::write(path, src).unwrap();
      });

    // Group of tests to be run.
    let tests = vec![
      "./core/tests/006_more_imports",
      "./core/tests/006_more_imports/",
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
  fn load_node_module2() {
    test_log_init();
    // Crate temp dir.
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
      export function sayHello() {
          console.log('Hello, World!');
      }
  "#;
    let src_file: &str = "./core/tests/006_more_imports/lib/index.js";

    let pkg: &str = r#"
      {
        "exports": {
          ".": "./lib/index.js"
        }
      }
    "#;
    let pkg_file: &str = "./core/tests/006_more_imports/package.json";

    // Create source files.
    [(src_file, src), (pkg_file, pkg)]
      .iter()
      .for_each(|(file, src)| {
        let path = Path::new(file);
        let path = temp_dir.child(path);

        path.touch().unwrap();
        fs::write(path, src).unwrap();
      });

    // Group of tests to be run.
    let tests = vec![
      "./core/tests/006_more_imports",
      "./core/tests/006_more_imports/",
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
  fn load_node_module_not_found1() {
    test_log_init();
    // Crate temp dir.
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
      export function sayHello() {
          console.log('Hello, World!');
      }
  "#;
    let src_file: &str = "./core/tests/006_more_imports/dist/index.js";

    let pkg: &str = r#"
      {
        "exports": "./lib/index.js"
      }
    "#;
    let pkg_file: &str = "./core/tests/006_more_imports/package.json";

    // Create source files.
    [(src_file, src), (pkg_file, pkg)]
      .iter()
      .for_each(|(file, src)| {
        let path = Path::new(file);
        let path = temp_dir.child(path);

        path.touch().unwrap();
        fs::write(path, src).unwrap();
      });

    // Group of tests to be run.
    let tests = vec![
      "./core/tests/006_more_imports",
      "./core/tests/006_more_imports/",
    ];

    // Run tests.
    let loader = FsModuleLoader {};

    for specifier in tests {
      let path = format!("{}", temp_dir.child(specifier).display());
      let source = loader.load(&path);
      info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
      assert!(source.is_err());
    }
  }
}

#[cfg(test)]
mod tests_async_filepath {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn load_files_dirs1() {
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
    let loader = AsyncFsModuleLoader {};

    for specifier in tests {
      let path = format!("{}", temp_dir.child(specifier).display());
      let source = loader.load(&path).await;
      info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
      assert!(source.is_ok());
      assert_eq!(source.unwrap(), src);
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn load_json1() {
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
    let loader = AsyncFsModuleLoader {};

    for specifier in tests {
      let path = format!("{}", temp_dir.child(specifier).display());
      let source = loader.load(&path).await;
      info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
      assert!(source.is_ok());
      assert!(source.unwrap().contains(src));
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn load_node_module1() {
    test_log_init();
    // Crate temp dir.
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
      export function sayHello() {
          console.log('Hello, World!');
      }
  "#;
    let src_file: &str = "./core/tests/006_more_imports/index.js";

    let pkg: &str = r#"
      {
        "exports": "./index.js"
      }
    "#;
    let pkg_file: &str = "./core/tests/006_more_imports/package.json";

    // Create source files.
    [(src_file, src), (pkg_file, pkg)]
      .iter()
      .for_each(|(file, src)| {
        let path = Path::new(file);
        let path = temp_dir.child(path);

        path.touch().unwrap();
        fs::write(path, src).unwrap();
      });

    // Group of tests to be run.
    let tests = vec![
      "./core/tests/006_more_imports",
      "./core/tests/006_more_imports/",
    ];

    // Run tests.
    let loader = AsyncFsModuleLoader {};

    for specifier in tests {
      let path = format!("{}", temp_dir.child(specifier).display());
      let source = loader.load(&path).await;
      info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
      assert!(source.is_ok());
      assert_eq!(source.unwrap(), src);
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn load_node_module2() {
    test_log_init();
    // Crate temp dir.
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
      export function sayHello() {
          console.log('Hello, World!');
      }
  "#;
    let src_file: &str = "./core/tests/006_more_imports/lib/index.js";

    let pkg: &str = r#"
      {
        "exports": {
          ".": "./lib/index.js"
        }
      }
    "#;
    let pkg_file: &str = "./core/tests/006_more_imports/package.json";

    // Create source files.
    [(src_file, src), (pkg_file, pkg)]
      .iter()
      .for_each(|(file, src)| {
        let path = Path::new(file);
        let path = temp_dir.child(path);

        path.touch().unwrap();
        fs::write(path, src).unwrap();
      });

    // Group of tests to be run.
    let tests = vec![
      "./core/tests/006_more_imports",
      "./core/tests/006_more_imports/",
    ];

    // Run tests.
    let loader = AsyncFsModuleLoader {};

    for specifier in tests {
      let path = format!("{}", temp_dir.child(specifier).display());
      let source = loader.load(&path).await;
      info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
      assert!(source.is_ok());
      assert_eq!(source.unwrap(), src);
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn load_node_module_not_found1() {
    test_log_init();
    // Crate temp dir.
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
      export function sayHello() {
          console.log('Hello, World!');
      }
  "#;
    let src_file: &str = "./core/tests/006_more_imports/lib/index.js";

    let pkg: &str = r#"
      {
        "exports": {
          ".": "./dist/index.js"
        }
      }
    "#;
    let pkg_file: &str = "./core/tests/006_more_imports/package.json";

    // Create source files.
    [(src_file, src), (pkg_file, pkg)]
      .iter()
      .for_each(|(file, src)| {
        let path = Path::new(file);
        let path = temp_dir.child(path);

        path.touch().unwrap();
        fs::write(path, src).unwrap();
      });

    // Group of tests to be run.
    let tests = vec![
      "./core/tests/006_more_imports",
      "./core/tests/006_more_imports/",
    ];

    // Run tests.
    let loader = AsyncFsModuleLoader {};

    for specifier in tests {
      let path = format!("{}", temp_dir.child(specifier).display());
      let source = loader.load(&path).await;
      info!("specifier:{specifier:?},path:{path:?},source:{source:?}");
      assert!(source.is_err());
    }
  }
}
