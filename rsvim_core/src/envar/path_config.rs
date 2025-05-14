//! File path configs.

use std::path::PathBuf;

#[derive(Debug, Clone)]
/// The configs for editor's config file, i.e. the `.rsvim.js` or `.rsvim.ts`.
pub struct PathConfig {
  config_file: Option<PathBuf>,
  config_dirs: Vec<PathBuf>,
  cache_dir: PathBuf,
  data_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct CachedDirs {
  config_dir: PathBuf,
  home_dir: PathBuf,
  cache_dir: PathBuf,
  data_dir: PathBuf,
}

/// For windows: `$env:USERPROFILE\AppData\Roaming\rsvim`.
/// For others: `$XDG_CONFIG_HOME/rsvim` or `$HOME/.config/rsvim`.
fn _xdg_config_dir(cached_dirs: &CachedDirs) -> PathBuf {
  cached_dirs.config_dir.join("rsvim").to_path_buf()
}

/// For all: `$HOME/.rsvim`.
fn _home_config_dir(cached_dirs: &CachedDirs) -> PathBuf {
  cached_dirs.home_dir.join(".rsvim")
}

/// Find the config file `rsvim.ts` or `rsvim.js` in rsvim config directory. This method will look
/// for the config file in 3 locations:
/// 1. The `$HOME/.rsvim.ts` or `$HOME/.rsvim.js`. This is similar to vim's `$HOME/.vimrc` config
///    file.
/// 2. The `$HOME/.rsvim/rsvim.ts` or `$HOME/.rsvim/rsvim.js`.
/// 3. The `$XDG_CONFIG_HOME/rsvim/rsvim.ts` or `$XDG_CONFIG_HOME/rsvim/rsvim.js`.
///
/// NOTE: The `ts` file always has higher priority.
fn get_config_file(cached_dirs: &CachedDirs) -> Option<PathBuf> {
  for config_dir in [_xdg_config_dir(cached_dirs), _home_config_dir(cached_dirs)].iter() {
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
    cached_dirs.home_dir.join(".rsvim.ts").to_path_buf(),
    cached_dirs.home_dir.join(".rsvim.js").to_path_buf(),
  ]
  .into_iter()
  .find(|p| p.exists())
}

fn get_config_dirs(cached_dirs: &CachedDirs) -> Vec<PathBuf> {
  vec![_xdg_config_dir(cached_dirs), _home_config_dir(cached_dirs)]
    .into_iter()
    .filter(|p| p.exists())
    .collect()
}

/// For windows: `$env:USERPROFILE\AppData\Local\rsvim-cache`.
/// For others: `$XDG_CACHE_HOME/rsvim` or `$HOME/.cache/rsvim`.
fn _xdg_cache_dir(cached_dirs: &CachedDirs) -> PathBuf {
  let folder = if cfg!(target_os = "windows") {
    "rsvim-cache"
  } else {
    "rsvim"
  };
  cached_dirs.cache_dir.join(folder).to_path_buf()
}

// For windows: `$env:USERPROFILE\AppData\Roaming\rsvim-data`.
// For others: `$XDG_DATA_HOME/rsvim` or `$HOME/.local/share/rsvim`.
fn _xdg_data_dir(cached_dirs: &CachedDirs) -> PathBuf {
  let folder = if cfg!(target_os = "windows") {
    "rsvim-data"
  } else {
    "rsvim"
  };
  cached_dirs.data_dir.join(folder).to_path_buf()
}

impl PathConfig {
  /// Make new path config.
  pub fn new() -> Self {
    let cached_dirs = CachedDirs {
      config_dir: dirs::config_dir().unwrap(),
      home_dir: dirs::home_dir().unwrap(),
      cache_dir: dirs::cache_dir().unwrap(),
      data_dir: dirs::data_dir().unwrap(),
    };
    PathConfig {
      config_file: get_config_file(&cached_dirs),
      config_dirs: get_config_dirs(&cached_dirs),
      cache_dir: _xdg_cache_dir(&cached_dirs),
      data_dir: _xdg_data_dir(&cached_dirs),
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
