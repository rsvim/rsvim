use super::path_config::*;
// use crate::test::log::init as test_log_init;

// use std::sync::Once;
use tracing::info;

// static INIT: Once = Once::new();

#[cfg(target_os = "windows")]
#[test]
fn config_file_windows() {
  // INIT.call_once(test_log_init);
  let cfg = PathConfig::default();
  match cfg.config_file().as_ref() {
    Some(actual) => {
      info!("config_file (windows): ${:?}", actual);
      assert!(
        actual.to_str().unwrap().ends_with(".rsvim.js")
          || actual.to_str().unwrap().ends_with(".rsvim.ts")
      );
    }
    None => { /* Skip */ }
  }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn config_file_unix() {
  // INIT.call_once(test_log_init);
  let cfg = PathConfig::default();
  match cfg.config_entry().as_ref() {
    Some(actual) => {
      info!("config_file (unix): ${:?}", actual);
      assert!(
        actual.to_str().unwrap().ends_with(".rsvim.js")
          || actual.to_str().unwrap().ends_with(".rsvim.ts")
      );
    }
    None => { /* Skip */ }
  }
}
