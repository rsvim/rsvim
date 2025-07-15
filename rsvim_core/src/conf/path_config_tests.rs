use super::path_config::*;
use crate::test::log::init as test_log_init;

use std::io::Write;
use tracing::info;

const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

macro_rules! save_xdg {
  ($name:ident, $value:expr) => {
    unsafe {
      let saved = std::env::var($name);
      std::env::set_var($name, $value);
      saved
    }
  };
}

macro_rules! restore_xdg {
  ($saved_value:ident, $name:ident) => {
    match $saved_value {
      Ok(saved) => unsafe {
        std::env::set_var($name, saved);
      },
      Err(_) => { /* do nothing */ }
    }
  };
}

#[test]
fn xdg_config_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_config_home = save_xdg!(XDG_CONFIG_HOME, tmpdir.path());

  std::fs::create_dir(tmpdir.path().join("rsvim")).unwrap();
  let mut config_entry = std::fs::File::create("rsvim.js").unwrap();
  info!("config_entry:{config_entry:?}");
  config_entry.write_all(b"hello").unwrap();

  let cfg = PathConfig::default();
  assert!(cfg.config_home().is_some());
  let actual1 = cfg.config_home().clone().unwrap();
  assert_eq!(actual1, tmpdir.path().join("rsvim"));

  assert!(cfg.config_entry().is_some());
  let actual2 = cfg.config_entry().clone().unwrap();
  assert_eq!(actual2, tmpdir.path().join("rsvim").join("rsvim.js"));

  restore_xdg!(saved_config_home, XDG_CONFIG_HOME);
}

#[test]
fn xdg_config_home2() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_config_home = save_xdg!(XDG_CONFIG_HOME, tmpdir.path());

  std::fs::create_dir(tmpdir.path().join("rsvim")).unwrap();
  let mut config_entry1 = std::fs::File::create("rsvim.js").unwrap();
  config_entry1.write_all(b"hello").unwrap();
  info!("config_entry1:{config_entry1:?}");
  let mut config_entry2 = std::fs::File::create("rsvim.ts").unwrap();
  config_entry2.write_all(b"hello").unwrap();
  info!("config_entry2:{config_entry2:?}");

  let cfg = PathConfig::default();
  assert!(cfg.config_home().is_some());
  let actual1 = cfg.config_home().clone().unwrap();
  assert_eq!(actual1, tmpdir.path().join("rsvim"));

  assert!(cfg.config_entry().is_some());
  let actual2 = cfg.config_entry().clone().unwrap();
  assert_eq!(actual2, tmpdir.path().join("rsvim").join("rsvim.ts"));

  restore_xdg!(saved_config_home, XDG_CONFIG_HOME);
}

#[cfg(target_os = "windows")]
#[test]
fn xdg_cache_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_cache_home = save_xdg!(XDG_CACHE_HOME, tmpdir.path());

  std::fs::create_dir(tmpdir.path().join("rsvim-cache")).unwrap();

  let cfg = PathConfig::default();
  let actual1 = cfg.cache_home().clone();
  assert_eq!(actual1, tmpdir.path().join("rsvim-cache"));

  restore_xdg!(saved_cache_home, XDG_CACHE_HOME);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn xdg_cache_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_cache_home = save_xdg!(XDG_CACHE_HOME, tmpdir.path());

  std::fs::create_dir(tmpdir.path().join("rsvim")).unwrap();

  let cfg = PathConfig::default();
  let actual1 = cfg.cache_home().clone();
  assert_eq!(actual1, tmpdir.path().join("rsvim"));

  restore_xdg!(saved_cache_home, XDG_CACHE_HOME);
}

#[cfg(target_os = "windows")]
#[test]
fn xdg_data_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_data_home = save_xdg!(XDG_DATA_HOME, tmpdir.path());

  std::fs::create_dir(tmpdir.path().join("rsvim-data")).unwrap();

  let cfg = PathConfig::default();
  let actual1 = cfg.data_home().clone();
  assert_eq!(actual1, tmpdir.path().join("rsvim-data"));

  restore_xdg!(saved_data_home, XDG_DATA_HOME);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn xdg_data_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  let saved_data_home = save_xdg!(XDG_DATA_HOME, tmpdir.path());

  std::fs::create_dir(tmpdir.path().join("rsvim")).unwrap();

  let cfg = PathConfig::default();
  let actual1 = cfg.data_home().clone();
  assert_eq!(actual1, tmpdir.path().join("rsvim"));

  restore_xdg!(saved_data_home, XDG_DATA_HOME);
}
