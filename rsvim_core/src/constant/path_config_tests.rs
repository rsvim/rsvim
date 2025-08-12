use super::path_config::*;

use crate::tests::constant::TempPathCfg;
use crate::tests::log::init as test_log_init;

use std::io::Write;

#[test]
fn test1() {
  test_log_init();

  let tp = TempPathCfg::create();

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
}

#[test]
fn test2() {
  test_log_init();

  let tp = TempPathCfg::create();

  // Prepare config home/entry
  {
    std::fs::create_dir_all(tp.home_dir.join(".rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tp.home_dir.join(".rsvim").join("rsvim.js"))
        .unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  let cfg = PathConfig::new();
  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    tp.home_dir.join(".rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    tp.home_dir.join(".rsvim").join("rsvim.js")
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
}

#[test]
fn test3() {
  test_log_init();

  let tp = TempPathCfg::create();

  // Prepare config home/entry
  {
    std::fs::create_dir_all(tp.home_dir.join(".rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tp.home_dir.join(".rsvim.js")).unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  let cfg = PathConfig::new();
  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    tp.home_dir.join(".rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    tp.home_dir.join(".rsvim.js")
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
}
