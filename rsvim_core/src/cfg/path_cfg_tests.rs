// use super::path_cfg::*;
use crate::evloop::mock::TempConfigDir;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use std::io::Write;

#[test]
fn test1() {
  test_log_init();

  let tp = TempConfigDir::create();

  // Prepare config home/entry
  {
    std::fs::create_dir_all(tp.xdg_config_home.join("rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tp.xdg_config_home.join("rsvim").join("rsvim.js"))
        .unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  assert_eq!(*PATH_CONFIG.config_home(), tp.xdg_config_home.join("rsvim"));

  assert!(PATH_CONFIG.config_entry().is_some());
  assert_eq!(
    PATH_CONFIG.config_entry().map(|e| e.to_path_buf()).unwrap(),
    tp.xdg_config_home.join("rsvim").join("rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(
      PATH_CONFIG.cache_home().to_path_buf(),
      tp.xdg_cache_home.join("rsvim-cache")
    );
    assert_eq!(
      PATH_CONFIG.data_home().to_path_buf(),
      tp.xdg_data_home.join("rsvim-data")
    );
  } else {
    assert_eq!(
      PATH_CONFIG.cache_home().to_path_buf(),
      tp.xdg_cache_home.join("rsvim")
    );
    assert_eq!(
      PATH_CONFIG.data_home().to_path_buf(),
      tp.xdg_data_home.join("rsvim")
    );
  }
}

#[test]
fn test2() {
  test_log_init();

  let tp = TempConfigDir::create();

  // Prepare config home/entry
  {
    std::fs::create_dir_all(tp.home_dir.join(".rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tp.home_dir.join(".rsvim").join("rsvim.js"))
        .unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  assert_eq!(*PATH_CONFIG.config_home(), tp.home_dir.join(".rsvim"));

  assert!(PATH_CONFIG.config_entry().is_some());
  assert_eq!(
    PATH_CONFIG.config_entry().map(|e| e.to_path_buf()).unwrap(),
    tp.home_dir.join(".rsvim").join("rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(
      PATH_CONFIG.cache_home().to_path_buf(),
      tp.xdg_cache_home.join("rsvim-cache")
    );
    assert_eq!(
      PATH_CONFIG.data_home().to_path_buf(),
      tp.xdg_data_home.join("rsvim-data")
    );
  } else {
    assert_eq!(
      PATH_CONFIG.cache_home().to_path_buf(),
      tp.xdg_cache_home.join("rsvim")
    );
    assert_eq!(
      PATH_CONFIG.data_home().to_path_buf(),
      tp.xdg_data_home.join("rsvim")
    );
  }
}

#[test]
fn test3() {
  test_log_init();

  let tp = TempConfigDir::create();

  // Prepare config home/entry
  {
    std::fs::create_dir_all(tp.home_dir.join(".rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tp.home_dir.join(".rsvim.js")).unwrap();
    config_entry.write_all(b"hello").unwrap();
    config_entry.flush().unwrap();
  }

  assert_eq!(*PATH_CONFIG.config_home(), tp.home_dir.join(".rsvim"));

  assert!(PATH_CONFIG.config_entry().is_some());
  assert_eq!(
    PATH_CONFIG.config_entry().map(|e| e.to_path_buf()).unwrap(),
    tp.home_dir.join(".rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(
      PATH_CONFIG.cache_home().to_path_buf(),
      tp.xdg_cache_home.join("rsvim-cache")
    );
    assert_eq!(
      PATH_CONFIG.data_home().to_path_buf(),
      tp.xdg_data_home.join("rsvim-data")
    );
  } else {
    assert_eq!(
      PATH_CONFIG.cache_home().to_path_buf(),
      tp.xdg_cache_home.join("rsvim")
    );
    assert_eq!(
      PATH_CONFIG.data_home().to_path_buf(),
      tp.xdg_data_home.join("rsvim")
    );
  }
}
