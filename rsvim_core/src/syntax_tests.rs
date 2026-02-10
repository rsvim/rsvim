use super::syntax::*;
use crate::cli::CliOptions;
use crate::prelude::*;
use crate::state::ops as state_ops;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use compact_str::CompactString;
use compact_str::ToCompactString;
use ropey::Rope;
use ropey::RopeBuilder;
use std::ops::Range;
use std::time::Duration;

#[cfg(test)]
mod tests_getter_setter {
  use super::*;

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
    assert_eq!(lang.unwrap().name(), Some("rust"));
  }
}

#[cfg(test)]
mod tests_buffer_editing {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn insert1() -> IoResult<()> {
    test_log_init();

    let src: &str = r#""#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![
      MockOperation::Operation(state_ops::Operation::GotoInsertMode(
        state_ops::GotoInsertModeVariant::Keep,
      )),
      MockOperation::Operation(state_ops::Operation::CursorInsert(
        state_ops::CursorInsertPayload::Text("Hello".to_compact_string()),
      )),
      MockOperation::Operation(state_ops::Operation::CursorInsert(
        state_ops::CursorInsertPayload::Text(", ".to_compact_string()),
      )),
      MockOperation::Operation(state_ops::Operation::CursorInsert(
        state_ops::CursorInsertPayload::Text("World".to_compact_string()),
      )),
      MockOperation::Operation(state_ops::Operation::GotoNormalMode),
      MockOperation::SleepFor(Duration::from_millis(30)),
    ];

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buf = lock!(event_loop.buffers)
        .first_key_value()
        .unwrap()
        .1
        .clone();
      let mut buf = lock!(buf);
      let buf_eol = buf.options().end_of_line();
      let after_payload = buf.text().rope().to_string();
      assert_eq!(after_payload, format!("Hello, World{}", buf_eol));
      let max_commits = buf.undo().undo_stack().len();
      debug_assert_eq!(max_commits, 1);
      let mut revert_rope = buf.text().rope().clone();
      buf.undo_mut().undo(0, &mut revert_rope).unwrap();
      let before_payload = revert_rope.to_string();
      assert_eq!(before_payload, format!("{}", buf_eol));
    }

    Ok(())
  }
}
