//! File path configs.

use std::path::PathBuf;

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
fn _home_config_dir(cached_dirs: &CachedDirs) -> PathBuf {
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
fn get_config_home_and_entry(cached_dirs: &CachedDirs) -> Option<ConfigHomeAndEntry> {
  for config_dir in [_xdg_config_dir(cached_dirs), _home_config_dir(cached_dirs)].iter() {
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
        config_home: _home_config_dir(cached_dirs),
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
