use std::io::Write;

use super::path_config::*;

use crate::test::log::init as test_log_init;

const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

macro_rules! set_xdg {
  ($name:ident,$value:expr) => {
    unsafe {
      let saved = std::env::var($name);
      std::env::set_var($name, $value);
      saved
    }
  };
}

macro_rules! restore_xdg {
  ($name:ident,$saved_value:ident) => {
    match $saved_value {
      Ok(saved) => unsafe { std::env::set_var($name, saved) },
      Err(_) => { /* */ }
    }
  };
}

fn xdg_config_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_xdg = set_xdg!(XDG_CONFIG_HOME, tmpdir.path());

  let cached_dirs = CachedDirs {
    config_dir: tmpdir.path().join("rsvim-config"),
    home_dir: tmpdir.path().join("rsvim-home"),
    cache_dir: tmpdir.path().join("rsvim-cache"),
    data_dir: tmpdir.path().join("rsvim-data"),
  };

  {
    let mut config_entry = std::fs::File::open("rsvim.js").unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  let cfg = PathConfig::_new_with_cached_dirs(&cached_dirs);
  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    cached_dirs.config_dir.join("rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    cached_dirs.config_dir.join("rsvim").join("rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(
      cfg.cache_home().clone(),
      cached_dirs.cache_dir.join("rsvim-cache")
    );
    assert_eq!(
      cfg.data_home().clone(),
      cached_dirs.data_dir.join("rsvim-data")
    );
  } else {
    assert_eq!(
      cfg.cache_home().clone(),
      cached_dirs.cache_dir.join("rsvim")
    );
    assert_eq!(cfg.data_home().clone(), cached_dirs.data_dir.join("rsvim"));
  }

  restore_xdg!(XDG_CONFIG_HOME, saved_xdg);
}

fn xdg_cache_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_xdg = set_xdg!(XDG_CACHE_HOME, tmpdir.path());

  let cached_dirs = CachedDirs {
    config_dir: tmpdir.path().join("rsvim-config"),
    home_dir: tmpdir.path().join("rsvim-home"),
    cache_dir: tmpdir.path().join("rsvim-cache"),
    data_dir: tmpdir.path().join("rsvim-data"),
  };

  {
    let mut config_entry = std::fs::File::open("rsvim.js").unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  let cfg = PathConfig::_new_with_cached_dirs(&cached_dirs);
  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    cached_dirs.config_dir.join("rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    cached_dirs.config_dir.join("rsvim").join("rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(
      cfg.cache_home().clone(),
      cached_dirs.cache_dir.join("rsvim-cache")
    );
    assert_eq!(
      cfg.data_home().clone(),
      cached_dirs.data_dir.join("rsvim-data")
    );
  } else {
    assert_eq!(
      cfg.cache_home().clone(),
      cached_dirs.cache_dir.join("rsvim")
    );
    assert_eq!(cfg.data_home().clone(), cached_dirs.data_dir.join("rsvim"));
  }

  restore_xdg!(XDG_CACHE_HOME, saved_xdg);
}

fn xdg_data_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_xdg = set_xdg!(XDG_DATA_HOME, tmpdir.path());

  let cached_dirs = CachedDirs {
    config_dir: tmpdir.path().join("rsvim-config"),
    home_dir: tmpdir.path().join("rsvim-home"),
    cache_dir: tmpdir.path().join("rsvim-cache"),
    data_dir: tmpdir.path().join("rsvim-data"),
  };

  {
    let mut config_entry = std::fs::File::open("rsvim.js").unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  let cfg = PathConfig::_new_with_cached_dirs(&cached_dirs);
  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    cached_dirs.config_dir.join("rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    cached_dirs.config_dir.join("rsvim").join("rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(
      cfg.cache_home().clone(),
      cached_dirs.cache_dir.join("rsvim-cache")
    );
    assert_eq!(
      cfg.data_home().clone(),
      cached_dirs.data_dir.join("rsvim-data")
    );
  } else {
    assert_eq!(
      cfg.cache_home().clone(),
      cached_dirs.cache_dir.join("rsvim")
    );
    assert_eq!(cfg.data_home().clone(), cached_dirs.data_dir.join("rsvim"));
  }

  restore_xdg!(XDG_DATA_HOME, saved_xdg);
}

#[test]
fn test_all() {
  xdg_config_home1();
  xdg_cache_home1();
  xdg_data_home1();
}
