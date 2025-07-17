use super::path_config::*;

use crate::test::log::init as test_log_init;

use std::io::Write;
use std::path::Path;

fn create_config_home_and_entry(cached_dirs: &CachedDirs) {
  std::fs::create_dir_all(cached_dirs.config_dir.join("rsvim")).unwrap();
  let mut config_entry =
    std::fs::File::create(cached_dirs.config_dir.join("rsvim").join("rsvim.js")).unwrap();
  config_entry.write_all(b"hello").unwrap();
  config_entry.flush().unwrap();
}

#[test]
fn xdg_config_home1() {
  test_log_init();

  let tmpdir = assert_fs::TempDir::new().unwrap();

  let cached_dirs = CachedDirs {
    config_dir: tmpdir.path().join("rsvim-config"),
    home_dir: tmpdir.path().join("rsvim-home"),
    cache_dir: tmpdir.path().join("rsvim-cache"),
    data_dir: tmpdir.path().join("rsvim-data"),
  };

  create_config_home_and_entry(&cached_dirs);

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
}
