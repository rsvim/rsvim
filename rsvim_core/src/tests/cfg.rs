pub struct TempPathCfg {
  pub home_dir: assert_fs::TempDir,
  pub xdg_config_home: assert_fs::TempDir,
  pub xdg_cache_home: assert_fs::TempDir,
  pub xdg_data_home: assert_fs::TempDir,
}

impl TempPathCfg {
  pub fn create() -> Self {
    let home_dir = assert_fs::TempDir::new().unwrap();
    let xdg_config_home = assert_fs::TempDir::new().unwrap();
    let xdg_cache_home = assert_fs::TempDir::new().unwrap();
    let xdg_data_home = assert_fs::TempDir::new().unwrap();

    Self {
      home_dir,
      xdg_config_home,
      xdg_cache_home,
      xdg_data_home,
    }
  }
}
