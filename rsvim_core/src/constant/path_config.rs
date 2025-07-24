//! File path configs.

use std::path::{Path, PathBuf};

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
fn get_config_home_and_entry(
  config_dir: &Path,
  home_dir: &Path,
) -> Option<(
  /* config_home */ PathBuf,
  /* config_entry */ PathBuf,
)> {
  for config_dir in [_xdg_config_dir(config_dir), _home_dir(home_dir)].iter() {
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
    home_dir.join(".rsvim.ts").to_path_buf(),
    home_dir.join(".rsvim.js").to_path_buf(),
  ]
  .iter()
  {
    if config_entry.exists() {
      return Some((_home_dir(home_dir), config_entry.clone()));
    }
  }

  None
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
    let config_dir = _dirs_config_dir().unwrap();
    let home_dir = _dirs_home_dir().unwrap();
    let cache_dir = _dirs_cache_dir().unwrap();
    let data_dir = _dirs_data_dir().unwrap();
    let config_home_and_entry =
      get_config_home_and_entry(&config_dir, &home_dir);
    Self {
      config_home: config_home_and_entry.as_ref().map(|c| c.0.clone()),
      config_entry: config_home_and_entry.as_ref().map(|c| c.1.clone()),
      cache_home: _xdg_cache_dir(&cache_dir),
      data_home: _xdg_data_dir(&data_dir),
    }
  }

  #[cfg(not(test))]
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

  #[cfg(test)]
  pub fn config_entry(&self) -> Option<PathBuf> {
    Self::new().config_entry.clone()
  }

  #[cfg(not(test))]
  /// User config home directory, it can be either one of following directories:
  ///
  /// 1. `$XDG_CONFIG_HOME/rsvim/` or `$HOME/.config/rsvim/`.
  /// 2. `$HOME/.rsvim/`
  pub fn config_home(&self) -> &Option<PathBuf> {
    &self.config_home
  }

  #[cfg(test)]
  pub fn config_home(&self) -> Option<PathBuf> {
    Self::new().config_home.clone()
  }

  #[cfg(not(test))]
  /// Cache home directory, i.e. `$XDG_CACHE_HOME/rsvim`.
  pub fn cache_home(&self) -> &PathBuf {
    &self.cache_home
  }

  #[cfg(test)]
  pub fn cache_home(&self) -> PathBuf {
    Self::new().cache_home.clone()
  }

  #[cfg(not(test))]
  /// Data home directory, i.e. `$XDG_DATA_HOME/rsvim`.
  pub fn data_home(&self) -> &PathBuf {
    &self.data_home
  }

  #[cfg(test)]
  pub fn data_home(&self) -> PathBuf {
    Self::new().data_home.clone()
  }
}

impl Default for PathConfig {
  fn default() -> Self {
    PathConfig::new()
  }
}
