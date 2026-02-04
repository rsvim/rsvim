//! Parsing syntax.

use crate::syntax::SyntaxEdit;
use parking_lot::Mutex;
use std::sync::Arc;
use tree_sitter::Parser;
use tree_sitter::Tree;

pub async fn parse_syntax(
  parser: Arc<Mutex<Parser>>,
  editing_version: isize,
  tree: Option<Tree>,
  pending_edits: Vec<SyntaxEdit>,
) {
}
