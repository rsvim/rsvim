//! Js config of config file path.

use directories::BaseDirs;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Clone)]
/// The configs for editor's config file, i.e. the `.rsvim.js` or `.rsvim.ts`.
pub struct PathConfig {
  config_file: Option<PathBuf>,
  cache_dir: PathBuf,
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

impl PathConfig {
  /// Make new path config.
  pub fn new() -> Self {
    if let Some(base_dirs) = BaseDirs::new() {
      let config_file = get_config_file(&base_dirs);
      let cache_dir = get_cache_dir(&base_dirs);
      PathConfig {
        config_file,
        cache_dir,
      }
    } else {
      unreachable!("Failed to find `$HOME` directory!")
    }
  }

  /// Get the config file in following directories and ts/js files.
  ///
  /// 1. $XDG_CONFIG_HOME/rsvim/rsvim.{ts,js}
  /// 2. $HOME/.rsvim/rsvim.{ts.js}
  /// 3. $HOME/.rsvim.{ts.js}
  ///
  /// NOTE:
  /// 1. If both `.ts` and `.js` files exist, prefer the `.ts` file.
  /// 2. For macOS, the `$XDG_CONFIG_HOME` also detects the `$HOME/.config` folder.
  pub fn config_file(&self) -> &Option<PathBuf> {
    &self.config_file
  }

  /// Get the cache directory.
  pub fn cache_dir(&self) -> &PathBuf {
    &self.cache_dir
  }
}

impl Default for PathConfig {
  fn default() -> Self {
    PathConfig::new()
  }
}
