//! File path configs.

use directories::BaseDirs;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Clone)]
/// The configs for editor's config file, i.e. the `.rsvim.js` or `.rsvim.ts`.
pub struct PathConfig {
  config_file: Option<PathBuf>,
  cache_dir: PathBuf,
  data_dir: PathBuf,
}

#[cfg(not(target_os = "macos"))]
fn _xdg_config_dirs(base_dirs: &BaseDirs) -> Vec<PathBuf> {
  vec![base_dirs.config_local_dir().join("rsvim").to_path_buf()]
}

#[cfg(target_os = "macos")]
fn _xdg_config_dirs(base_dirs: &BaseDirs) -> Vec<PathBuf> {
  vec![
    base_dirs.config_local_dir().join("rsvim").to_path_buf(),
    base_dirs.home_dir().join(".config").join("rsvim"),
  ]
}

fn _home_config_dirs(base_dirs: &BaseDirs) -> Vec<PathBuf> {
  vec![base_dirs.home_dir().join(".rsvim")]
}

fn _ts_config_file(config_dir: &Path) -> PathBuf {
  config_dir.join("rsvim.ts")
}

fn _js_config_file(config_dir: &Path) -> PathBuf {
  config_dir.join("rsvim.js")
}

fn get_config_file(base_dirs: &BaseDirs) -> Option<PathBuf> {
  let mut xdg_dirs = _xdg_config_dirs(base_dirs);
  debug!("xdg config dirs:{:?}", xdg_dirs);

  let mut home_dirs = _home_config_dirs(base_dirs);
  debug!("home config dirs:{:?}", home_dirs);

  xdg_dirs.append(&mut home_dirs);

  for xdg_dir in xdg_dirs.iter() {
    let xdg_ts_path = _ts_config_file(xdg_dir);
    if xdg_ts_path.as_path().exists() {
      return Some(xdg_ts_path);
    }
    let xdg_js_path = _js_config_file(xdg_dir);
    if xdg_js_path.as_path().exists() {
      return Some(xdg_js_path);
    }
  }

  let home_paths = vec![
    base_dirs.home_dir().join(".rsvim.ts").to_path_buf(),
    base_dirs.home_dir().join(".rsvim.js").to_path_buf(),
  ];
  home_paths.into_iter().find(|p| p.exists())
}

fn get_cache_dir(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs.cache_dir().join("rsvim").to_path_buf()
}

#[cfg(not(target_os = "macos"))]
fn _xdg_data_dirs(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs.config_local_dir().join("rsvim").to_path_buf()
}

#[cfg(target_os = "macos")]
fn _xdg_data_dirs(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs
    .home_dir()
    .join(".local")
    .join("share")
    .join("rsvim")
}

fn get_data_dir(base_dirs: &BaseDirs) -> PathBuf {
  _xdg_data_dirs(base_dirs)
}

impl PathConfig {
  /// Make new path config.
  pub fn new() -> Self {
    let base_dirs = BaseDirs::new().unwrap();
    let config_file = get_config_file(&base_dirs);
    let cache_dir = get_cache_dir(&base_dirs);
    let data_dir = get_data_dir(&base_dirs);
    PathConfig {
      config_file,
      cache_dir,
      data_dir,
    }
  }

  /// Get the config file.
  pub fn config_file(&self) -> &Option<PathBuf> {
    &self.config_file
  }

  /// Get the cache directory.
  pub fn cache_dir(&self) -> &PathBuf {
    &self.cache_dir
  }

  /// Get the data directory.
  pub fn data_dir(&self) -> &PathBuf {
    &self.data_dir
  }
}

impl Default for PathConfig {
  fn default() -> Self {
    PathConfig::new()
  }
}
