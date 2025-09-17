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

#[cfg(test)]
mod tests_sync {
  use super::*;

  #[test]
  fn file_path1() {
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

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn file_path2() {
    test_log_init();
    let tp = TempPathCfg::create();
    let temp_dir = assert_fs::TempDir::new().unwrap();

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

    let base = temp_dir.child("core/tests");
    let specifier = "./006_more_imports.js";
    let expect = temp_dir.child("core/tests/006_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader::new();

    // Prepare configs
    {
      // base.touch().unwrap();
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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn file_path3() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let base = temp_dir.child("core/tests/");
    let specifier = "../006_more_imports.js";
    let expect = temp_dir.child("core/006_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader::new();

    // Prepare configs
    {
      // base.touch().unwrap();
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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn file_path4() {
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
    let loader = FsModuleLoader::new();

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn file_path_failed5() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = temp_dir.child("core/tests/006_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader::new();

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let specifier = transform(specifier.to_path_buf());

    let actual = loader.resolve(base, &specifier);
    assert!(actual.is_err());
  }

  #[test]
  fn file_path6() {
    test_log_init();
    let tp = TempPathCfg::create();

    let src: &str = r#"
export function sayHello() {
    console.log('Hello, World!');
}
"#;

    let entry = tp.xdg_config_home.child("rsvim").child("rsvim.js");
    let specifier = "006_more_imports.js";
    let expect = tp
      .xdg_config_home
      .child("rsvim")
      .child("006_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader::new();

    // Prepare configs
    {
      entry.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
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
  fn folder_path1() {
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
    let loader = FsModuleLoader::new();

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn folder_path2() {
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
    let loader = FsModuleLoader::new();

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn folder_path3() {
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
    let loader = FsModuleLoader::new();

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn folder_path4() {
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
    let loader = FsModuleLoader::new();

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn folder_path_failed5() {
    test_log_init();
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let base = temp_dir.child("core/tests/005_more_imports.js");
    let specifier = temp_dir.child("core/tests/006_more_imports/");

    // Run tests.
    let loader = FsModuleLoader::new();

    // Prepare configs
    {
      base.touch().unwrap();
      // expect.touch().unwrap();
      // fs::write(expect.path(), src).unwrap();
    }

    let base: Option<&str> = Some(base.as_os_str().to_str().unwrap());
    let specifier = transform(specifier.to_path_buf());

    let actual = loader.resolve(base, &specifier);
    assert!(actual.is_err());
  }

  #[test]
  fn npm_package1() {
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
    let loader = FsModuleLoader::new();

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[test]
  fn npm_package2() {
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
    let loader = FsModuleLoader::new();

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = loader.load(&actual);
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
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
}

#[cfg(test)]
mod tests_async {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn file_path1() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn file_path2() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn file_path3() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn file_path4() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
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

    let entry = tp.xdg_config_home.child("rsvim").child("rsvim.js");
    let specifier = "006_more_imports.js";
    let expect = tp
      .xdg_config_home
      .child("rsvim")
      .child("006_more_imports.js");

    // Run tests.
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

    // Prepare configs
    {
      entry.touch().unwrap();
      expect.touch().unwrap();
      fs::write(expect.path(), src).unwrap();
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

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn folder_path1() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn folder_path2() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn folder_path3() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn folder_path4() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn npm_package1() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn npm_package2() {
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
    let loader = FsModuleLoader::new();
    let aloader = AsyncFsModuleLoader {};

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
    assert_eq!(
      Path::new(&actual).normalize().unwrap(),
      Path::new(&expect).normalize().unwrap()
    );

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn npm_package3() {
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
    let aloader = AsyncFsModuleLoader {};

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

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn npm_package4() {
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
    let aloader = AsyncFsModuleLoader {};

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

    let actual_module = aloader.load(&actual).await;
    assert!(actual_module.is_ok());
    assert_eq!(actual_module.unwrap(), src);
  }
}
