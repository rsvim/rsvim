use super::syntax::*;
use compact_str::ToCompactString;

#[test]
fn file_ext1() {
  let mut syn_mgr = SyntaxManager::new();
  syn_mgr.insert_file_ext(LanguageId::from("rust".to_compact_string()), "rs");
  let actual = syn_mgr.get_id_by_file_ext("rs");
  assert!(actual.is_some());
  assert_eq!(actual.unwrap(), &LanguageId::from("rust"));
  assert_eq!(actual.unwrap(), &LanguageId::from("rust".to_string()));
  assert_eq!(
    actual.unwrap(),
    &LanguageId::from("rust".to_compact_string())
  );
  let actual = syn_mgr.get_file_ext_by_id(&LanguageId::from("rust"));
  assert!(actual.is_some());
  assert!(actual.unwrap().contains("rs"));
}

#[test]
fn file_ext2() {
  let mut syn_mgr = SyntaxManager::new();
  syn_mgr.insert_file_ext(LanguageId::from("cpp".to_compact_string()), "cc");
  syn_mgr.insert_file_ext(LanguageId::from("cpp".to_compact_string()), "cpp");
  syn_mgr.insert_file_ext(LanguageId::from("cpp".to_compact_string()), "c++");
  syn_mgr.insert_file_ext(LanguageId::from("cpp".to_compact_string()), "hh");
  syn_mgr.insert_file_ext(LanguageId::from("cpp".to_compact_string()), "hpp");
  syn_mgr.insert_file_ext(LanguageId::from("cpp".to_compact_string()), "h++");
  let actual = syn_mgr.get_id_by_file_ext("hpp");
  assert!(actual.is_some());
  assert_eq!(actual.unwrap(), &LanguageId::from("cpp"));
  assert_eq!(actual.unwrap(), &LanguageId::from("cpp".to_string()));
  assert_eq!(
    actual.unwrap(),
    &LanguageId::from("cpp".to_compact_string())
  );
  let actual = syn_mgr.get_file_ext_by_id(&LanguageId::from("cpp"));
  assert!(actual.is_some());
  assert!(actual.unwrap().contains("cc"));
  assert!(actual.unwrap().contains("cpp"));
  assert!(actual.unwrap().contains("c++"));
  assert!(actual.unwrap().contains("hh"));
  assert!(actual.unwrap().contains("hpp"));
  assert!(actual.unwrap().contains("h++"));
}

#[test]
fn get_lang1() {
  let mut syn_mgr = SyntaxManager::new();
  syn_mgr.insert_file_ext(LanguageId::from("rust".to_compact_string()), "rs");
  let lang = syn_mgr.get_lang_by_ext("rs");
  assert!(lang.is_some());
  assert_eq!(lang.unwrap().name(), "rust");
}
