use super::path_config::*;

use crate::tests::constant::TempPaths;
use crate::tests::log::init as test_log_init;

use std::io::Write;

#[test]
fn test1() {
  test_log_init();

  let tp = TempPaths::create();

  // Prepare config home/entry
  {
    std::fs::create_dir_all(tp.xdg_config_home.join("rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tp.xdg_config_home.join("rsvim").join("rsvim.js"))
        .unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  let cfg = PathConfig::new();
  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    tp.xdg_config_home.join("rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    tp.xdg_config_home.join("rsvim").join("rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(
      cfg.cache_home().clone(),
      tp.xdg_cache_home.join("rsvim-cache")
    );
    assert_eq!(cfg.data_home().clone(), tp.xdg_data_home.join("rsvim-data"));
  } else {
    assert_eq!(cfg.cache_home().clone(), tp.xdg_cache_home.join("rsvim"));
    assert_eq!(cfg.data_home().clone(), tp.xdg_data_home.join("rsvim"));
  }

  tp.restore();
}

#[test]
fn test2() {
  let _guard = acquire_sequential_guard();
  test_log_init();

  let tmp_home_dir = assert_fs::TempDir::new().unwrap();
  let tmp_config_dir = assert_fs::TempDir::new().unwrap();
  let tmp_cache_dir = assert_fs::TempDir::new().unwrap();
  let tmp_data_dir = assert_fs::TempDir::new().unwrap();

  let saved_home = set_env_var(HOME, tmp_home_dir.path());
  let saved_conf = set_env_var(XDG_CONFIG_HOME, tmp_config_dir.path());
  let saved_cache = set_env_var(XDG_CACHE_HOME, tmp_cache_dir.path());
  let saved_data = set_env_var(XDG_DATA_HOME, tmp_data_dir.path());

  // Prepare config home/entry
  {
    std::fs::create_dir_all(tmp_home_dir.join(".rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tmp_home_dir.join(".rsvim").join("rsvim.js"))
        .unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  let cfg = PathConfig::new();
  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    tmp_home_dir.join(".rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    tmp_home_dir.join(".rsvim").join("rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(cfg.cache_home().clone(), tmp_cache_dir.join("rsvim-cache"));
    assert_eq!(cfg.data_home().clone(), tmp_data_dir.join("rsvim-data"));
  } else {
    assert_eq!(cfg.cache_home().clone(), tmp_cache_dir.join("rsvim"));
    assert_eq!(cfg.data_home().clone(), tmp_data_dir.join("rsvim"));
  }

  restore_env_var(HOME, saved_home);
  restore_env_var(XDG_CONFIG_HOME, saved_conf);
  restore_env_var(XDG_CACHE_HOME, saved_cache);
  restore_env_var(XDG_DATA_HOME, saved_data);
}

#[test]
fn test3() {
  let _guard = acquire_sequential_guard();
  test_log_init();

  let tmp_home_dir = assert_fs::TempDir::new().unwrap();
  let tmp_config_dir = assert_fs::TempDir::new().unwrap();
  let tmp_cache_dir = assert_fs::TempDir::new().unwrap();
  let tmp_data_dir = assert_fs::TempDir::new().unwrap();

  let saved_home = set_env_var(HOME, tmp_home_dir.path());
  let saved_conf = set_env_var(XDG_CONFIG_HOME, tmp_config_dir.path());
  let saved_cache = set_env_var(XDG_CACHE_HOME, tmp_cache_dir.path());
  let saved_data = set_env_var(XDG_DATA_HOME, tmp_data_dir.path());

  // Prepare config home/entry
  {
    std::fs::create_dir_all(tmp_home_dir.join(".rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tmp_home_dir.join(".rsvim.js")).unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  let cfg = PathConfig::new();
  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    tmp_home_dir.join(".rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    tmp_home_dir.join(".rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(cfg.cache_home().clone(), tmp_cache_dir.join("rsvim-cache"));
    assert_eq!(cfg.data_home().clone(), tmp_data_dir.join("rsvim-data"));
  } else {
    assert_eq!(cfg.cache_home().clone(), tmp_cache_dir.join("rsvim"));
    assert_eq!(cfg.data_home().clone(), tmp_data_dir.join("rsvim"));
  }

  restore_env_var(HOME, saved_home);
  restore_env_var(XDG_CONFIG_HOME, saved_conf);
  restore_env_var(XDG_CACHE_HOME, saved_cache);
  restore_env_var(XDG_DATA_HOME, saved_data);
}
