//! File path configs.

use directories::BaseDirs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
/// The configs for editor's config file, i.e. the `.rsvim.js` or `.rsvim.ts`.
pub struct PathConfig {
  config_file: Option<PathBuf>,
  cache_dir: PathBuf,
  data_dir: PathBuf,
}

// `$env:LocalAppData\rsvim`
#[cfg(target_os = "windows")]
fn _xdg_config_dir(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs.config_local_dir().join("rsvim").to_path_buf()
}

// `$XDG_CONFIG_HOME/rsvim` or `$HOME/.config/rsvim`
#[cfg(not(target_os = "windows"))]
fn _xdg_config_dir(base_dirs: &BaseDirs) -> PathBuf {
  match std::env::var("XDG_CONFIG_HOME") {
    Ok(config_path) => Path::new(&config_path).join("rsvim").to_path_buf(),
    Err(_) => base_dirs.home_dir().join(".config").join("rsvim"),
  }
}

// `$HOME/.rsvim`
fn _home_config_dir(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs.home_dir().join(".rsvim")
}

fn get_config_file(base_dirs: &BaseDirs) -> Option<PathBuf> {
  for config_dir in [_xdg_config_dir(base_dirs), _home_config_dir(base_dirs)].iter() {
    let ts_config = config_dir.join("rsvim.ts");
    if ts_config.as_path().exists() {
      return Some(ts_config);
    }
    let js_config = config_dir.join("rsvim.js");
    if js_config.as_path().exists() {
      return Some(js_config);
    }
  }

  // `$HOME/.rsvim.js` or `$HOME/.rsvim.ts`
  vec![
    base_dirs.home_dir().join(".rsvim.ts").to_path_buf(),
    base_dirs.home_dir().join(".rsvim.js").to_path_buf(),
  ]
  .into_iter()
  .find(|p| p.exists())
}

fn get_cache_dir(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs.cache_dir().join("rsvim").to_path_buf()
}

/// `$env:LocalAppData\rsvim-data`
#[cfg(target_os = "windows")]
fn _xdg_data_dir(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs.data_local_dir().join("rsvim-data").to_path_buf()
}

/// `$XDG_DATA_HOME/rsvim` or `$HOME/.local/share/rsvim`
#[cfg(not(target_os = "windows"))]
fn _xdg_data_dir(base_dirs: &BaseDirs) -> PathBuf {
  match std::env::var("XDG_DATA_HOME") {
    Ok(data_path) => Path::new(&data_path).join("rsvim").to_path_buf(),
    Err(_) => base_dirs
      .home_dir()
      .join(".local")
      .join("share")
      .join("rsvim"),
  }
}

fn get_data_dir(base_dirs: &BaseDirs) -> PathBuf {
  _xdg_data_dir(base_dirs)
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
