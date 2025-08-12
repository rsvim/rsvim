use parking_lot::{Mutex, MutexGuard};
use std::env::VarError;
use std::ffi::OsStr;

static GLOBAL_SEQUENTIAL_LOCK: Mutex<()> = Mutex::new(());

// pub fn acquire_sequential_guard() -> MutexGuard<'static, ()> {
//   GLOBAL_SEQUENTIAL_LOCK.lock()
// }

fn set_env_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(
  name: K,
  value: V,
) -> Result<String, VarError> {
  let saved = std::env::var(&name);
  unsafe {
    std::env::set_var(name, value);
  }
  saved
}

fn restore_env_var(name: &str, saved_value: Result<String, VarError>) {
  match saved_value {
    Ok(saved) => unsafe {
      std::env::set_var(name, saved);
    },
    Err(_) => { /* */ }
  }
}

pub struct TempPathCfg {
  pub home_dir: assert_fs::TempDir,
  pub xdg_config_home: assert_fs::TempDir,
  pub xdg_cache_home: assert_fs::TempDir,
  pub xdg_data_home: assert_fs::TempDir,

  saved_home_dir: Option<Result<String, VarError>>,
  saved_xdg_config_home: Option<Result<String, VarError>>,
  saved_xdg_cache_home: Option<Result<String, VarError>>,
  saved_xdg_data_home: Option<Result<String, VarError>>,

  sequential_guard: MutexGuard<'static, ()>,
}

impl TempPathCfg {
  pub fn create() -> Self {
    use crate::constant::path_config::{
      HOME, XDG_CACHE_HOME, XDG_CONFIG_HOME, XDG_DATA_HOME,
    };

    let home_dir = assert_fs::TempDir::new().unwrap();
    let xdg_config_home = assert_fs::TempDir::new().unwrap();
    let xdg_cache_home = assert_fs::TempDir::new().unwrap();
    let xdg_data_home = assert_fs::TempDir::new().unwrap();

    let saved_home_dir = set_env_var(HOME, home_dir.path());
    let saved_xdg_config_home =
      set_env_var(XDG_CONFIG_HOME, xdg_config_home.path());
    let saved_xdg_cache_home =
      set_env_var(XDG_CACHE_HOME, xdg_cache_home.path());
    let saved_xdg_data_home = set_env_var(XDG_DATA_HOME, xdg_data_home.path());

    Self {
      home_dir,
      xdg_config_home,
      xdg_cache_home,
      xdg_data_home,
      saved_home_dir: Some(saved_home_dir),
      saved_xdg_config_home: Some(saved_xdg_config_home),
      saved_xdg_cache_home: Some(saved_xdg_cache_home),
      saved_xdg_data_home: Some(saved_xdg_data_home),
      sequential_guard: GLOBAL_SEQUENTIAL_LOCK.lock(),
    }
  }
}

impl Drop for TempPathCfg {
  fn drop(&mut self) {
    use crate::constant::path_config::{
      HOME, XDG_CACHE_HOME, XDG_CONFIG_HOME, XDG_DATA_HOME,
    };

    let saved_home_dir = self.saved_home_dir.take().unwrap();
    let saved_xdg_config_home = self.saved_xdg_config_home.take().unwrap();
    let saved_xdg_cache_home = self.saved_xdg_cache_home.take().unwrap();
    let saved_xdg_data_home = self.saved_xdg_data_home.take().unwrap();
    restore_env_var(HOME, saved_home_dir);
    restore_env_var(XDG_CONFIG_HOME, saved_xdg_config_home);
    restore_env_var(XDG_CACHE_HOME, saved_xdg_cache_home);
    restore_env_var(XDG_DATA_HOME, saved_xdg_data_home);
  }
}
