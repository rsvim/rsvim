use super::path_config::*;

use crate::test::log::init as test_log_init;

use std::io::Write;
use std::path::Path;

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
  ($name:ident,$saved_xdg:expr) => {
    match $saved_xdg {
      Ok(saved) => unsafe {
        std::env::set_var($name, saved);
      },
      Err(_) => { /* */ }
    }
  };
}

fn create_config_home_and_entry(tmpdir: &Path) {
  std::fs::create_dir_all(tmpdir.join("rsvim")).unwrap();
  let mut config_entry = std::fs::File::create(tmpdir.join("rsvim").join("rsvim.js")).unwrap();
  config_entry.write_all(b"hello").unwrap();
  config_entry.flush().unwrap();
}

fn xdg_variables1() {
  test_log_init();

  let tmp_conf = assert_fs::TempDir::new().unwrap();
  let tmp_cache = assert_fs::TempDir::new().unwrap();
  let tmp_data = assert_fs::TempDir::new().unwrap();

  let saved_xdg_config = set_xdg!(XDG_CONFIG_HOME, tmp_conf.path());
  let saved_xdg_cache = set_xdg!(XDG_CACHE_HOME, tmp_cache.path());
  let saved_xdg_data = set_xdg!(XDG_DATA_HOME, tmp_data.path());

  create_config_home_and_entry(tmp_conf.path());

  let cfg = PathConfig::default();

  assert!(cfg.config_home().is_some());
  assert_eq!(
    cfg.config_home().clone().unwrap(),
    tmp_conf.path().join("rsvim")
  );

  assert!(cfg.config_entry().is_some());
  assert_eq!(
    cfg.config_entry().clone().unwrap(),
    tmp_conf.path().join("rsvim").join("rsvim.js")
  );

  if cfg!(target_os = "windows") {
    assert_eq!(
      cfg.cache_home().clone(),
      tmp_cache.path().join("rsvim-cache")
    );
    assert_eq!(cfg.data_home().clone(), tmp_data.path().join("rsvim-data"));
  } else {
    assert_eq!(cfg.cache_home().clone(), tmp_cache.path().join("rsvim"));
    assert_eq!(cfg.data_home().clone(), tmp_data.path().join("rsvim"));
  }

  restore_xdg!(XDG_CONFIG_HOME, saved_xdg_config);
  restore_xdg!(XDG_CACHE_HOME, saved_xdg_cache);
  restore_xdg!(XDG_DATA_HOME, saved_xdg_data);
}

#[test]
fn test_all() {
  xdg_variables1();
}
