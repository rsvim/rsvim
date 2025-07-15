use super::path_config::*;

use crate::test::log::init as test_log_init;

use tracing::info;

#[test]
fn xdg_config_home1() {
  test_log_init();

  let expect1 = dirs::config_dir().unwrap().join("rsvim");
  let expect2 = dirs::home_dir().unwrap().join(".rsvim");

  let cfg = PathConfig::default();
  let actual1 = cfg.config_home().is_some();
  let actual2 = cfg.config_entry().is_some();
  assert_eq!(actual1, actual2);

  match cfg.config_home() {
    Some(actual3) => {
      info!("config_home, actual3:{actual3:?}, expect1:{expect1:?}, expect2:{expect2:?}");
      assert!(actual3.clone() == expect1 || actual3.clone() == expect2);
    }
    None => { /* */ }
  }
  match cfg.config_entry() {
    Some(actual4) => {
      info!("config_entry, actual4:{actual4:?}, expect1:{expect1:?}, expect2:{expect2:?}");
      assert!(
        actual4
          .clone()
          .to_str()
          .unwrap()
          .starts_with(expect1.to_str().unwrap())
          || actual4
            .clone()
            .to_str()
            .unwrap()
            .starts_with(expect2.to_str().unwrap())
      );
    }
    None => { /* */ }
  }
}

#[cfg(target_os = "windows")]
#[test]
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
#[test]
fn xdg_cache_home1_unix() {
  test_log_init();

  let cfg = PathConfig::default();
  let actual = cfg.cache_home();
  info!("cache_home:{actual:?}");
  assert_eq!(actual.clone(), dirs::cache_dir().unwrap().join("rsvim"));
}

#[cfg(target_os = "windows")]
#[test]
fn xdg_data_home1_win() {
  test_log_init();

  let cfg = PathConfig::default();
  let actual = cfg.data_home().clone();
  info!("data_home:{actual:?}");
  assert_eq!(actual.clone(), dirs::data_dir().unwrap().join("rsvim-data"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn xdg_data_home1_unix() {
  test_log_init();

  let cfg = PathConfig::default();
  let actual = cfg.data_home().clone();
  info!("data_home:{actual:?}");
  assert_eq!(actual.clone(), dirs::data_dir().unwrap().join("rsvim"));
}
