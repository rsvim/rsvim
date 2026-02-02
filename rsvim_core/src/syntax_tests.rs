use super::syntax::*;

#[test]
fn init1() {
  let mut syn_mgr = SyntaxManager::new();
  syn_mgr.insert_lang_id_and_file_ext(LanguageId::Rust, "rs");
}
