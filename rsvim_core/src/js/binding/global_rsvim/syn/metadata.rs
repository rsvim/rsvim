//! Tree-sitter parser metadata

use crate::js::converter::*;
use crate::to_v8_prop;
use compact_str::CompactString;
use compact_str::ToCompactString;

pub const NAME: &str = "name";
pub const CAMELCASE: &str = "camelcase";
pub const SCOPE: &str = "scope";
pub const PATH: &str = "scope";
pub const FILE_TYPES: &str = "fileTypes";
pub const HIGHLIGHTS_PATH: &str = "highlightsPath";
pub const HIGHLIGHTS_QUERY: &str = "highlightsQuery";
pub const TAGS_PATH: &str = "tagsPath";
pub const TAGS_QUERY: &str = "tagsQuery";
pub const INJECTIONS_PATH: &str = "injectionsPath";
pub const INJECTIONS_QUERY: &str = "injectionsQuery";
pub const INJECTION_REGEX: &str = "injectionRegex";

// Defaults
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

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
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

impl StructToV8 for SynTreeSitterParserMetadata {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, name);
    to_v8_prop!(self, obj, scope, camelcase);
    to_v8_prop!(self, obj, scope, scope);
    to_v8_prop!(self, obj, scope, path);
    to_v8_prop!(self, obj, scope, file_types, Vec);
    to_v8_prop!(self, obj, scope, highlights_path, optional);
    to_v8_prop!(self, obj, scope, highlights_query, optional);
    to_v8_prop!(self, obj, scope, tags_path, optional);
    to_v8_prop!(self, obj, scope, tags_query, optional);
    to_v8_prop!(self, obj, scope, injections_path, optional);
    to_v8_prop!(self, obj, scope, injections_query, optional);
    to_v8_prop!(self, obj, scope, injection_regex, optional);

    obj
  }
}
