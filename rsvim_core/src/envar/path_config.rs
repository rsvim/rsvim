//! File path configs.

use std::path::PathBuf;

#[derive(Debug, Clone)]
/// The configs for editor's config file, i.e. the `.rsvim.js` or `.rsvim.ts`.
pub struct PathConfig {
  config_entry: Option<PathBuf>,
  config_home: Option<PathBuf>,
  cache_home: PathBuf,
  data_home: PathBuf,
}

#[derive(Debug, Clone)]
struct CachedDirs {
  config_dir: PathBuf,
  home_dir: PathBuf,
  cache_dir: PathBuf,
  data_dir: PathBuf,
}

/// `$XDG_CONFIG_HOME/rsvim`
fn _xdg_config_dir(cached_dirs: &CachedDirs) -> PathBuf {
  cached_dirs.config_dir.join("rsvim").to_path_buf()
}

/// `$HOME/.rsvim`
fn _home_config_dir(cached_dirs: &CachedDirs) -> PathBuf {
  cached_dirs.home_dir.join(".rsvim")
}

/// Find the config home directory. This method look for config entry (`rsvim.{js,ts}`) in below
/// locations:
/// 1. `$XDG_CONFIG_HOME/rsvim`
/// 2. `$HOME/.rsvim`, additionally, `$HOME/.rsvim.{js,ts}` will be recognized as config entry as
///    well.
///
/// It returns `(Home, Entry)`.
fn get_config_home_and_entry(cached_dirs: &CachedDirs) -> Option<(PathBuf, PathBuf)> {
  for config_dir in [_xdg_config_dir(cached_dirs), _home_config_dir(cached_dirs)].iter() {
    let ts_config = config_dir.join("rsvim.ts");
    if ts_config.as_path().exists() {
      return Some((config_dir.to_path_buf(), ts_config));
    }
    let js_config = config_dir.join("rsvim.js");
    if js_config.as_path().exists() {
      return Some((config_dir.to_path_buf(), js_config));
    }
  }

  // `$HOME/.rsvim.js` or `$HOME/.rsvim.ts`
  for config_entry in [
    cached_dirs.home_dir.join(".rsvim.ts").to_path_buf(),
    cached_dirs.home_dir.join(".rsvim.js").to_path_buf(),
  ]
  .iter()
  {
    if config_entry.exists() {
      return Some((cached_dirs.home_dir.clone(), config_entry.clone()));
    }
  }

  None
}

fn get_config_home(cached_dirs: &CachedDirs) -> Option<PathBuf> {
  get_config_home_and_entry(cached_dirs).map(|c| c.0)
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
    let config_home_and_entry = get_config_home_and_entry(&cached_dirs);
    PathConfig {
      config_home: config_home_and_entry.as_ref().map(|c| c.0.clone()),
      config_entry: config_home_and_entry.as_ref().map(|c| c.1.clone()),
      cache_home: _xdg_cache_dir(&cached_dirs),
      data_home: _xdg_data_dir(&cached_dirs),
    }
  }

  /// Get the config entry file.
  pub fn config_entry(&self) -> &Option<PathBuf> {
    &self.config_entry
  }

  /// Get the config home directory.
  pub fn config_home(&self) -> &Option<PathBuf> {
    &self.config_home
  }

  /// Get the cache home directory.
  pub fn cache_home(&self) -> &PathBuf {
    &self.cache_home
  }

  /// Get the data home directory.
  pub fn data_home(&self) -> &PathBuf {
    &self.data_home
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
    match cfg.config_entry().as_ref() {
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
