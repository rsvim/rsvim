use crate::cli::CliOptions;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use ringbuf::traits::*;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_encode1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let src: &str = r#"
  queueMicrotask(() => {
    Rsvim.cmd.echo(1);
  });
"#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert!(n == 1);
    let actual = contents.command_line_message_history_mut().try_pop();
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert_eq!(actual, "1");
  }

  Ok(())
}
