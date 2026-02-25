use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attribute;
use crossterm::style::Attributes;
use crossterm::style::Color;
use once_cell::sync::Lazy;

// "ui."
pub const FOREGROUND: &str = "foreground";
pub const BACKGROUND: &str = "background";
pub const UI_FOREGROUND: &str = "ui.foreground";
pub const UI_BACKGROUND: &str = "ui.background";

// "scope.{lang}."
pub const ATTRIBUTE: &str = "attribute";
pub const BOOLEAN: &str = "boolean";
pub const CARRIAGE_RETURN: &str = "carriage-return";
pub const COMMENT: &str = "comment";
pub const COMMENT_DOCUMENTATION: &str = "comment.documentation";
pub const CONSTANT: &str = "constant";
pub const CONSTANT_BUILTIN: &str = "constant.builtin";
pub const CONSTRUCTOR: &str = "constructor";
pub const CONSTRUCTOR_BUILTIN: &str = "constructor.builtin";
pub const EMBEDDED: &str = "embedded";
pub const ERROR: &str = "error";
pub const ESCAPE: &str = "escape";
pub const FUNCTION: &str = "function";
pub const FUNCTION_BUILTIN: &str = "function.builtin";
pub const KEYWORD: &str = "keyword";
pub const MARKUP: &str = "markup";
pub const MARKUP_BOLD: &str = "markup.bold";
pub const MARKUP_HEADING: &str = "markup.heading";
pub const MARKUP_ITALIC: &str = "markup.italic";
pub const MARKUP_LINK: &str = "markup.link";
pub const MARKUP_LINK_URL: &str = "markup.link.url";
pub const MARKUP_LIST: &str = "markup.list";
pub const MARKUP_LIST_CHECKED: &str = "markup.list.checked";
pub const MARKUP_LIST_NUMBERED: &str = "markup.list.numbered";
pub const MARKUP_LIST_UNCHECKED: &str = "markup.list.unchecked";
pub const MARKUP_LIST_UNNUMBERED: &str = "markup.list.unnumbered";
pub const MARKUP_QUOTE: &str = "markup.quote";
pub const MARKUP_RAW: &str = "markup.raw";
pub const MARKUP_RAW_BLOCK: &str = "markup.raw.block";
pub const MARKUP_RAW_INLINE: &str = "markup.raw.inline";
pub const MARKUP_STRIKETHROUGH: &str = "markup.strikethrough";
pub const MODULE: &str = "module";
pub const NUMBER: &str = "number";
pub const OPERATOR: &str = "operator";
pub const PROPERTY: &str = "property";
pub const PROPERTY_BUILTIN: &str = "property.builtin";
pub const PUNCTUATION: &str = "punctuation";
pub const PUNCTUATION_BRACKET: &str = "punctuation.bracket";
pub const PUNCTUATION_DELIMITER: &str = "punctuation.delimiter";
pub const PUNCTUATION_SPECIAL: &str = "punctuation.special";
pub const STRING: &str = "string";
pub const STRING_ESCAPE: &str = "string.escape";
pub const STRING_REGEXP: &str = "string.regexp";
pub const STRING_SPECIAL: &str = "string.special";
pub const STRING_SPECIAL_SYMBOL: &str = "string.special.symbol";
pub const TAG: &str = "tag";
pub const TYPE: &str = "type";
pub const TYPE_BUILTIN: &str = "type.builtin";
pub const VARIABLE: &str = "variable";
pub const VARIABLE_BUILTIN: &str = "variable.builtin";
pub const VARIABLE_MEMBER: &str = "variable.member";
pub const VARIABLE_PARAMETER: &str = "variable.parameter";

#[derive(Debug, Clone, PartialEq, Eq)]
/// Highlight style, including colors and attributes.
pub struct Highlight {
  pub fg: Option<Color>,
  pub bg: Option<Color>,
  pub attr: Attributes,
}

pub struct Scope {}

pub struct Ui {}

pub struct Palette {}

pub struct Data {
  pub scope: Option<Scope>,
  pub ui: Option<Ui>,
  pub palette: Option<Palette>,
}
