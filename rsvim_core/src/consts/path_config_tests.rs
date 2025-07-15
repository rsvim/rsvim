use super::path_config::*;

use crate::test::log::init as test_log_init;

use tracing::info;

#[test]
fn xdg_config_home1() {
  test_log_init();

  let cfg = PathConfig::default();
  assert_eq!(cfg.config_home().is_some(), cfg.config_entry().is_some());

  match cfg.config_home() {
    Some(actual1) => {
      info!("config_home:{actual1:?}");
      assert_eq!(actual1.clone(), dirs::config_dir().unwrap().join("rsvim"));
    }
    None => { /* */ }
  }
  match cfg.config_entry() {
    Some(actual2) => {
      info!("config_entry:{actual2:?}");
      assert!(
        actual2
          .clone()
          .starts_with(dirs::config_dir().unwrap().join("rsvim"))
      );
    }
    None => { /* */ }
  }
}

#[cfg(target_os = "windows")]
#[cfg(test)]
fn xdg_cache_home1_win() {
  test_log_init();

  let cfg = PathConfig::default();
  let actual = cfg.cache_home();
  info!("cache_home:{actual:?}");
  assert_eq!(
    actual.clone(),
    dirs::cache_dir().unwrap().join("rsvim-cache")
  );
}

#[cfg(not(target_os = "windows"))]
#[cfg(test)]
fn xdg_cache_home1_unix() {
  test_log_init();

  let cfg = PathConfig::default();
  let actual = cfg.cache_home();
  info!("cache_home:{actual:?}");
  assert_eq!(actual.clone(), dirs::cache_dir().unwrap().join("rsvim"));
}

#[cfg(target_os = "windows")]
#[cfg(test)]
fn xdg_data_home1_win() {
  test_log_init();

  let cfg = PathConfig::default();
  let actual = cfg.data_home().clone();
  info!("data_home:{actual:?}");
  assert_eq!(actual.clone(), dirs::data_dir().unwrap().join("rsvim-data"));
}

#[cfg(not(target_os = "windows"))]
#[cfg(test)]
fn xdg_data_home1_unix() {
  test_log_init();

  let cfg = PathConfig::default();
  let actual = cfg.data_home().clone();
  info!("data_home:{actual:?}");
  assert_eq!(actual.clone(), dirs::data_dir().unwrap().join("rsvim-data"));
}
