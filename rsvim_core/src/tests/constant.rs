use crate::constant::path_config::XDG_VAR;
use crate::constant::path_config::XdgVar;
use fslock::LockFile;

const LOCK_FILE_NAME: &str = ".test.lock";

pub struct TempPathCfg {
  pub home_dir: assert_fs::TempDir,
  pub xdg_config_home: assert_fs::TempDir,
  pub xdg_cache_home: assert_fs::TempDir,
  pub xdg_data_home: assert_fs::TempDir,
  lock_file: LockFile,
}

impl TempPathCfg {
  pub fn create() -> Self {
    let mut lock_file = LockFile::open(LOCK_FILE_NAME).unwrap();
    lock_file.lock().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(5));

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
      lock_file,
    }
  }
}

impl Drop for TempPathCfg {
  fn drop(&mut self) {
    let mut var = (*XDG_VAR).lock();
    *var = None;

    self.lock_file.unlock().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
  }
}
