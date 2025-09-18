use crate::constant::path_config::XDG_VAR;
use crate::constant::path_config::XdgVar;

use parking_lot::Mutex;
use parking_lot::MutexGuard;

static GLOBAL_SEQUENTIAL_LOCK: Mutex<()> = Mutex::new(());

pub struct TempPathCfg {
  pub home_dir: assert_fs::TempDir,
  pub xdg_config_home: assert_fs::TempDir,
  pub xdg_cache_home: assert_fs::TempDir,
  pub xdg_data_home: assert_fs::TempDir,
  _sequential_guard: MutexGuard<'static, ()>,
}

impl TempPathCfg {
  pub fn create() -> Self {
    let _sequential_guard = GLOBAL_SEQUENTIAL_LOCK.lock();

    let home_dir = assert_fs::TempDir::new().unwrap();
    let xdg_config_home = assert_fs::TempDir::new().unwrap();
    let xdg_cache_home = assert_fs::TempDir::new().unwrap();
    let xdg_data_home = assert_fs::TempDir::new().unwrap();

    let mut var = (*XDG_VAR).lock();
    *var = Some(XdgVar {
      home_dir: home_dir.to_path_buf(),
      xdg_config_home_dir: xdg_config_home.to_path_buf(),
      xdg_cache_home_dir: xdg_cache_home.to_path_buf(),
      xdg_data_home_dir: xdg_data_home.to_path_buf(),
    });

    Self {
      home_dir,
      xdg_config_home,
      xdg_cache_home,
      xdg_data_home,
      _sequential_guard,
    }
  }
}

impl Drop for TempPathCfg {
  fn drop(&mut self) {
    let mut var = (*XDG_VAR).lock();
    *var = None;
  }
}
