use crate::constant::path_config::PATH_CONFIG_FILE;
use crate::prelude::*;
use fslock::LockFile;
use std::io::Write;

const LOCK_FILE_NAME: &str = ".test.lock";
const BLOCKING_MILLIS: u64 = 5;

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
    std::thread::sleep(std::time::Duration::from_millis(BLOCKING_MILLIS));

    let home_dir = assert_fs::TempDir::new().unwrap();
    let xdg_config_home = assert_fs::TempDir::new().unwrap();
    let xdg_cache_home = assert_fs::TempDir::new().unwrap();
    let xdg_data_home = assert_fs::TempDir::new().unwrap();

    let data = format!(
      "home_dir={}\nxdg_config_home_dir={}\nxdg_cache_home_dir={}\nxdg_data_home_dir={}",
      home_dir.path().to_string_lossy(),
      xdg_config_home.path().to_string_lossy(),
      xdg_cache_home.path().to_string_lossy(),
      xdg_data_home.path().to_string_lossy()
    );
    info!("TempPathCfg data:{:?}", data);

    let mut fp = std::fs::File::create(PATH_CONFIG_FILE).unwrap();
    fp.write_all(&data.into_bytes()).unwrap();
    fp.flush().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(BLOCKING_MILLIS));

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
    std::thread::sleep(std::time::Duration::from_millis(BLOCKING_MILLIS));
    std::fs::remove_file(PATH_CONFIG_FILE).unwrap();

    self.lock_file.unlock().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(BLOCKING_MILLIS));
  }
}
