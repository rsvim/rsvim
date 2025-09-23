//! File path configs.

use std::path::Path;
use std::path::PathBuf;

#[cfg(test)]
use crate::tests::evloop::TempPathConfig;

pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
pub const HOME: &str = "HOME";
pub const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

fn _dirs_config_dir() -> Option<PathBuf> {
  dirs::config_dir()
}

fn _dirs_home_dir() -> Option<PathBuf> {
  dirs::home_dir()
}

fn _dirs_cache_dir() -> Option<PathBuf> {
  dirs::cache_dir()
}

fn _dirs_data_dir() -> Option<PathBuf> {
  dirs::data_dir()
}

/// `$XDG_CONFIG_HOME/rsvim`
fn _xdg_config_dir(config_dir: &Path) -> PathBuf {
  config_dir.join("rsvim").to_path_buf()
}

/// `$HOME/.rsvim`
fn _home_dir(home_dir: &Path) -> PathBuf {
  home_dir.join(".rsvim")
}

/// Find the config home directory. This method look for config entry (`rsvim.{js,ts}`) in below
/// locations:
/// 1. `$XDG_CONFIG_HOME/rsvim`
/// 2. `$HOME/.rsvim`, additionally, `$HOME/.rsvim.{js,ts}` will be recognized as config entry as
///    well.
///
/// It returns `(Home, Entry)`.
fn find_config_home_and_entry(
  config_dir: &Path,
  home_dir: &Path,
) -> (
  /* config_home */ PathBuf,
  /* config_entry */ Option<PathBuf>,
) {
  for config_dir in [_xdg_config_dir(config_dir), _home_dir(home_dir)].iter() {
    let ts_config = config_dir.join("rsvim.ts");
    if ts_config.as_path().exists() {
      return (config_dir.to_path_buf(), Some(ts_config));
    }
    let js_config = config_dir.join("rsvim.js");
    if js_config.as_path().exists() {
      return (config_dir.to_path_buf(), Some(js_config));
    }
  }

  // `$HOME/.rsvim.js` or `$HOME/.rsvim.ts`
  for config_entry in [
    home_dir.join(".rsvim.ts").to_path_buf(),
    home_dir.join(".rsvim.js").to_path_buf(),
  ]
  .iter()
  {
    if config_entry.exists() {
      return (_home_dir(home_dir), Some(config_entry.clone()));
    }
  }

  // Config home fallback to `$HOME/.rsvim` even it doesn't exist, config entry
  // fallback to `None`.
  (_home_dir(home_dir), None)
}

/// For windows: `$env:USERPROFILE\AppData\Local\rsvim-cache`.
/// For others: `$XDG_CACHE_HOME/rsvim` or `$HOME/.cache/rsvim`.
fn _xdg_cache_dir(cache_dir: &Path) -> PathBuf {
  let folder = if cfg!(target_os = "windows") {
    "rsvim-cache"
  } else {
    "rsvim"
  };
  cache_dir.join(folder).to_path_buf()
}

// For windows: `$env:USERPROFILE\AppData\Roaming\rsvim-data`.
// For others: `$XDG_DATA_HOME/rsvim` or `$HOME/.local/share/rsvim`.
fn _xdg_data_dir(data_dir: &Path) -> PathBuf {
  let folder = if cfg!(target_os = "windows") {
    "rsvim-data"
  } else {
    "rsvim"
  };
  data_dir.join(folder).to_path_buf()
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// File path related configs.
pub struct PathConfig {
  config_entry: Option<PathBuf>,
  config_home: PathBuf,
  cache_home: PathBuf,
  data_home: PathBuf,
}

impl PathConfig {
  fn _new_internal(
    config_dir: PathBuf,
    home_dir: PathBuf,
    cache_dir: PathBuf,
    data_dir: PathBuf,
  ) -> Self {
    let config_home_and_entry =
      find_config_home_and_entry(&config_dir, &home_dir);
    Self {
      config_home: config_home_and_entry.0,
      config_entry: config_home_and_entry.1,
      cache_home: _xdg_cache_dir(&cache_dir),
      data_home: _xdg_data_dir(&data_dir),
    }
  }

  /// Make new path config.
  pub fn new() -> Self {
    let config_dir = _dirs_config_dir().unwrap();
    let home_dir = _dirs_home_dir().unwrap();
    let cache_dir = _dirs_cache_dir().unwrap();
    let data_dir = _dirs_data_dir().unwrap();
    Self::_new_internal(config_dir, home_dir, cache_dir, data_dir)
  }

  #[cfg(test)]
  pub fn _new_with_temp_dirs(tp: &TempPathConfig) -> Self {
    Self::_new_internal(
      tp.xdg_config_home.to_path_buf(),
      tp.home_dir.to_path_buf(),
      tp.xdg_cache_home.to_path_buf(),
      tp.xdg_data_home.to_path_buf(),
    )
  }

  #[cfg(not(test))]
  /// User config entry path, it can be either one of following files:
  ///
  /// 1. `$XDG_CONFIG_HOME/rsvim/rsvim.{ts,js}`
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

  #[cfg(test)]
  pub fn config_entry(&self) -> Option<PathBuf> {
    use crate::tests::evloop::TEMP_PATH_CONFIG;
    (*TEMP_PATH_CONFIG).lock().config_entry().clone()
  }

  #[cfg(not(test))]
  /// User config home directory, it can be either one of following directories:
  ///
  /// 1. `$XDG_CONFIG_HOME/rsvim/`
  /// 2. `$HOME/.rsvim/`
  pub fn config_home(&self) -> &PathBuf {
    &self.config_home
  }

  #[cfg(test)]
  pub fn config_home(&self) -> PathBuf {
    use crate::tests::evloop::TEMP_PATH_CONFIG;
    (*TEMP_PATH_CONFIG).lock().config_home().clone()
  }

  #[cfg(not(test))]
  /// Cache home directory, i.e. `$XDG_CACHE_HOME/rsvim`.
  pub fn cache_home(&self) -> &PathBuf {
    &self.cache_home
  }

  #[cfg(test)]
  pub fn cache_home(&self) -> PathBuf {
    use crate::tests::evloop::TEMP_PATH_CONFIG;
    (*TEMP_PATH_CONFIG).lock().cache_home().clone()
  }

  #[cfg(not(test))]
  /// Data home directory, i.e. `$XDG_DATA_HOME/rsvim`.
  pub fn data_home(&self) -> &PathBuf {
    &self.data_home
  }

  #[cfg(test)]
  pub fn data_home(&self) -> PathBuf {
    use crate::tests::evloop::TEMP_PATH_CONFIG;
    (*TEMP_PATH_CONFIG).lock().data_home().clone()
  }
}

impl Default for PathConfig {
  fn default() -> Self {
    PathConfig::new()
  }
}
