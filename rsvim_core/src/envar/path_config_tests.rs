use super::path_config::*;
use crate::test::log::init as test_log_init;

use std::io::Write;
use tracing::info;

#[cfg(not(target_os = "windows"))]
#[test]
fn make_xdg_cache_dir() {}

#[test]
fn xdg_config_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", tmpdir.path().to_path_buf());
  }
  std::fs::create_dir(tmpdir.path().join("rsvim"));
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
}

#[test]
fn xdg_config_home2() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", tmpdir.path().to_path_buf());
  }
  std::fs::create_dir(tmpdir.path().join("rsvim"));
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
}
