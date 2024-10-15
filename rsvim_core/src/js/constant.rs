//! Js global constants.

#![allow(non_snake_case)]

use regex::Regex;
use std::sync::OnceLock;

/// Full path regex on Windows platform.
pub fn WINDOWS_REGEX() -> Regex {
  static VALUE: OnceLock<Regex> = OnceLock::new();
  VALUE
    .get_or_init(|| Regex::new(r"^[a-zA-Z]:\\").unwrap())
    .clone()
}

/// URL regex validator (string begins with http:// or https://).
pub fn URL_REGEX() -> Regex {
  static VALUE: OnceLock<Regex> = OnceLock::new();
  VALUE
    .get_or_init(|| Regex::new(r"^(http|https)://").unwrap())
    .clone()
}
