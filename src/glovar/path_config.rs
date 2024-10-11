//! File path configs.

use directories::BaseDirs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
/// The configs for editor's config file, i.e. the `.rsvim.js` or `.rsvim.ts`.
pub struct PathConfig {
  config_file: Option<PathBuf>,
  config_dirs: Vec<PathBuf>,
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
    Ok(config_path) => std::path::Path::new(&config_path)
      .join("rsvim")
      .to_path_buf(),
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

fn get_config_dirs(base_dirs: &BaseDirs) -> Vec<PathBuf> {
  vec![_xdg_config_dir(base_dirs), _home_config_dir(base_dirs)]
    .into_iter()
    .filter(|p| p.exists())
    .collect()
}

// `$env:LocalAppData\rsvim-cache`
#[cfg(target_os = "windows")]
fn _xdg_cache_dir(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs.cache_dir().join("rsvim-cache").to_path_buf()
}

// `$XDG_CACHE_HOME/rsvim` or `$HOME/.cache/rsvim`
#[cfg(not(target_os = "windows"))]
fn _xdg_cache_dir(base_dirs: &BaseDirs) -> PathBuf {
  match std::env::var("XDG_CACHE_HOME") {
    Ok(cache_path) => std::path::Path::new(&cache_path)
      .join("rsvim")
      .to_path_buf(),
    Err(_) => base_dirs.home_dir().join(".cache").join("rsvim"),
  }
}

// For windows: `$env:`
fn get_cache_dir(base_dirs: &BaseDirs) -> PathBuf {
  _xdg_cache_dir(base_dirs)
}

// `$env:LocalAppData\rsvim-data`
#[cfg(target_os = "windows")]
fn _xdg_data_dir(base_dirs: &BaseDirs) -> PathBuf {
  base_dirs.data_local_dir().join("rsvim-data").to_path_buf()
}

// `$XDG_DATA_HOME/rsvim` or `$HOME/.local/share/rsvim`
#[cfg(not(target_os = "windows"))]
fn _xdg_data_dir(base_dirs: &BaseDirs) -> PathBuf {
  match std::env::var("XDG_DATA_HOME") {
    Ok(data_path) => std::path::Path::new(&data_path).join("rsvim").to_path_buf(),
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
    let config_dirs = get_config_dirs(&base_dirs);
    let cache_dir = get_cache_dir(&base_dirs);
    let data_dir = get_data_dir(&base_dirs);
    PathConfig {
      config_file,
      config_dirs,
      cache_dir,
      data_dir,
    }
  }

  /// Get the config file.
  pub fn config_file(&self) -> &Option<PathBuf> {
    &self.config_file
  }

  /// Get the config dirs.
  pub fn config_dirs(&self) -> &Vec<PathBuf> {
    &self.config_dirs
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

#[cfg(test)]
mod tests {
  use super::*;
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
    match cfg.config_file().as_ref() {
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
}
