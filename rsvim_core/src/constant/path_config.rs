//! File path configs.

use crate::prelude::*;

use std::path::PathBuf;

#[cfg(test)]
use std::path::Path;

pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
pub const HOME: &str = "HOME";
pub const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

#[cfg(test)]
fn _dirs_config_dir() -> Option<PathBuf> {
  match std::env::var(XDG_CONFIG_HOME) {
    Ok(config_home) => Some(Path::new(&config_home).to_path_buf()),
    Err(_) => None,
  }
}

#[cfg(not(test))]
fn _dirs_config_dir() -> Option<PathBuf> {
  dirs::config_dir()
}

#[cfg(test)]
fn _dirs_home_dir() -> Option<PathBuf> {
  match std::env::var(HOME) {
    Ok(home) => Some(Path::new(&home).to_path_buf()),
    Err(_) => None,
  }
}

#[cfg(not(test))]
fn _dirs_home_dir() -> Option<PathBuf> {
  dirs::home_dir()
}

#[cfg(test)]
fn _dirs_cache_dir() -> Option<PathBuf> {
  match std::env::var(XDG_CACHE_HOME) {
    Ok(cache_home) => Some(Path::new(&cache_home).to_path_buf()),
    Err(_) => None,
  }
}

#[cfg(not(test))]
fn _dirs_cache_dir() -> Option<PathBuf> {
  dirs::cache_dir()
}

#[cfg(test)]
fn _dirs_data_dir() -> Option<PathBuf> {
  match std::env::var(XDG_DATA_HOME) {
    Ok(data_home) => Some(Path::new(&data_home).to_path_buf()),
    Err(_) => None,
  }
}

#[cfg(not(test))]
fn _dirs_data_dir() -> Option<PathBuf> {
  dirs::data_dir()
}

#[derive(Debug, Clone)]
pub struct CachedDirs {
  pub config_dir: PathBuf,
  pub home_dir: PathBuf,
  pub cache_dir: PathBuf,
  pub data_dir: PathBuf,
}

/// `$XDG_CONFIG_HOME/rsvim`
fn _xdg_config_dir(cached_dirs: &CachedDirs) -> PathBuf {
  cached_dirs.config_dir.join("rsvim").to_path_buf()
}

/// `$HOME/.rsvim`
fn _home_dir(cached_dirs: &CachedDirs) -> PathBuf {
  cached_dirs.home_dir.join(".rsvim")
}

#[derive(Debug, Clone)]
struct ConfigHomeAndEntry {
  pub config_home: PathBuf,
  pub config_entry: PathBuf,
}

/// Find the config home directory. This method look for config entry (`rsvim.{js,ts}`) in below
/// locations:
/// 1. `$XDG_CONFIG_HOME/rsvim`
/// 2. `$HOME/.rsvim`, additionally, `$HOME/.rsvim.{js,ts}` will be recognized as config entry as
///    well.
///
/// It returns `(Home, Entry)`.
fn get_config_home_and_entry(
  cached_dirs: &CachedDirs,
) -> Option<ConfigHomeAndEntry> {
  for config_dir in
    [_xdg_config_dir(cached_dirs), _home_dir(cached_dirs)].iter()
  {
    let ts_config = config_dir.join("rsvim.ts");
    if ts_config.as_path().exists() {
      return Some(ConfigHomeAndEntry {
        config_home: config_dir.to_path_buf(),
        config_entry: ts_config,
      });
    }
    let js_config = config_dir.join("rsvim.js");
    if js_config.as_path().exists() {
      return Some(ConfigHomeAndEntry {
        config_home: config_dir.to_path_buf(),
        config_entry: js_config,
      });
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
      return Some(ConfigHomeAndEntry {
        config_home: _home_dir(cached_dirs),
        config_entry: config_entry.clone(),
      });
    }
  }

  None
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

#[derive(Debug, Clone)]
/// File path related configs.
pub struct PathConfig {
  config_entry: Option<PathBuf>,
  config_home: Option<PathBuf>,
  cache_home: PathBuf,
  data_home: PathBuf,
}

arc_ptr!(PathConfig);

impl PathConfig {
  /// Make new path config.
  pub fn new() -> Self {
    let cached_dirs = CachedDirs {
      config_dir: dirs::config_dir().unwrap(),
      home_dir: dirs::home_dir().unwrap(),
      cache_dir: dirs::cache_dir().unwrap(),
      data_dir: dirs::data_dir().unwrap(),
    };
    Self::_new_with_cached_dirs(&cached_dirs)
  }

  /// Internal constructor.
  pub fn _new_with_cached_dirs(cached_dirs: &CachedDirs) -> Self {
    let config_home_and_entry = get_config_home_and_entry(cached_dirs);
    Self {
      config_home: config_home_and_entry
        .as_ref()
        .map(|c| c.config_home.clone()),
      config_entry: config_home_and_entry
        .as_ref()
        .map(|c| c.config_entry.clone()),
      cache_home: _xdg_cache_dir(cached_dirs),
      data_home: _xdg_data_dir(cached_dirs),
    }
  }

  /// User config entry path, it can be either one of following files:
  ///
  /// 1. `$XDG_CONFIG_HOME/rsvim/rsvim.{ts,js}` or `$HOME/.config/rsvim/rsvim.{ts.js}`.
  /// 2. `$HOME/.rsvim/rsvim.{ts.js}`
  /// 3. `$HOME/.rsvim.{ts.js}`
  ///
  /// NOTE:
  /// 1. Typescript file is preferred over javascript, if both exist.
  /// 2. The detect priority is from higher to lower: 1st > 2nd > 3rd.
  /// 3. The 1st config home is `$XDG_CONFIG_HOME/rsvim`, the 2nd and 3rd config home is
  ///    `$HOME/.rsvim`.
  pub fn config_entry(&self) -> &Option<PathBuf> {
    &self.config_entry
  }

  /// User config home directory, it can be either one of following directories:
  ///
  /// 1. `$XDG_CONFIG_HOME/rsvim/` or `$HOME/.config/rsvim/`.
  /// 2. `$HOME/.rsvim/`
  pub fn config_home(&self) -> &Option<PathBuf> {
    &self.config_home
  }

  /// Cache home directory, i.e. `$XDG_CACHE_HOME/rsvim`.
  pub fn cache_home(&self) -> &PathBuf {
    &self.cache_home
  }

  /// Data home directory, i.e. `$XDG_DATA_HOME/rsvim`.
  pub fn data_home(&self) -> &PathBuf {
    &self.data_home
  }
}

impl Default for PathConfig {
  fn default() -> Self {
    PathConfig::new()
  }
}
