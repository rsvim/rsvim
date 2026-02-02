use super::syntax::*;
use compact_str::ToCompactString;

#[test]
fn init1() {
  let mut syn_mgr = SyntaxManager::new();
  syn_mgr.insert_lang_id_and_file_ext(
    LanguageId::from("rust".to_compact_string()),
    "rs",
  );
  let actual = syn_mgr.get_lang_id_by_file_ext("rs");
  assert!(actual.is_some());
  assert_eq!(actual.unwrap(), &LanguageId::from("rust"));
  assert_eq!(actual.unwrap(), &LanguageId::from("rust".to_string()));
  assert_eq!(
    actual.unwrap(),
    &LanguageId::from("rust".to_compact_string())
  );
}
