//! Tree-sitter parser metadata

use compact_str::CompactString;
use compact_str::ToCompactString;

// Default values
pub const NAME_DEFAULT: &str = "";
pub const CAMELCASE_DEFAULT: &str = "";
pub const SCOPE_DEFAULT: &str = "";
pub const PATH_DEFAULT: &str = "";
pub const FILE_TYPES_DEFAULT: Vec<CompactString> = vec![];
pub const HIGHLIGHTS_PATH_DEFAULT: Option<CompactString> = None;
pub const HIGHLIGHTS_QUERY_DEFAULT: Option<String> = None;
pub const TAGS_PATH_DEFAULT: Option<CompactString> = None;
pub const TAGS_QUERY_DEFAULT: Option<String> = None;
pub const INJECTIONS_PATH_DEFAULT: Option<CompactString> = None;
pub const INJECTIONS_QUERY_DEFAULT: Option<String> = None;
pub const INJECTION_REGEX_DEFAULT: Option<String> = None;

#[derive(
  Debug, Clone, PartialEq, Eq, derive_builder::Builder, rsvim_macro::ToV8,
)]
pub struct SynTreeSitterParserMetadata {
  #[builder(default = NAME_DEFAULT.to_compact_string())]
  pub name: CompactString,

  #[builder(default = CAMELCASE_DEFAULT.to_compact_string())]
  pub camelcase: CompactString,

  #[builder(default = SCOPE_DEFAULT.to_compact_string())]
  pub scope: CompactString,

  #[builder(default = PATH_DEFAULT.to_compact_string())]
  pub path: CompactString,

  #[builder(default = FILE_TYPES_DEFAULT)]
  pub file_types: Vec<CompactString>,

  #[builder(default = HIGHLIGHTS_PATH_DEFAULT)]
  pub highlights_path: Option<CompactString>,

  #[builder(default = HIGHLIGHTS_QUERY_DEFAULT)]
  pub highlights_query: Option<String>,

  #[builder(default = TAGS_PATH_DEFAULT)]
  pub tags_path: Option<CompactString>,

  #[builder(default = TAGS_QUERY_DEFAULT)]
  pub tags_query: Option<String>,

  #[builder(default = INJECTIONS_PATH_DEFAULT)]
  pub injections_path: Option<CompactString>,

  #[builder(default = INJECTIONS_QUERY_DEFAULT)]
  pub injections_query: Option<String>,

  #[builder(default = INJECTION_REGEX_DEFAULT)]
  pub injection_regex: Option<String>,
}
