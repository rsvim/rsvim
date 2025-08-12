use super::timeout::*;

use crate::constant::path_config::*;
use crate::js::loader::ModuleLoader;
use crate::prelude::*;
use crate::tests::constant::{
  acquire_sequential_guard, restore_env_var, set_env_var,
};
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;

use assert_fs::prelude::*;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[test]
fn test_timeout1() {
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

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(200))];
  let src: &str = r#"
  // Set timeout for 100 milliseconds.
  const timerId = setTimeout(() => {
    Rsvim.opt.wrap = false;
    Rsvim.opt.lineBreak = true;
  }, 100);
"#;

  let mut evloop = make_event_loop();
}
