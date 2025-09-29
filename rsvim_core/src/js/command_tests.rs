// use super::command::*;
use crate::cli::CliOptions;
use crate::evloop::mock::*;
use crate::prelude::*;
use crate::results::IoResult;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::tests::log::init as test_log_init;
use compact_str::ToCompactString;
use ringbuf::traits::*;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_js_echo1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("js Rsvim.cmd.echo(1);".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
    MockOperation::SleepFor(Duration::from_millis(50)),
  ];

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), "")]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert_eq!(actual, "1");
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_js_throw1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
  throw new Error(1);
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    ._run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);
    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert!(actual.contains("Uncaught Error: 1"));
  }

  Ok(())
}
