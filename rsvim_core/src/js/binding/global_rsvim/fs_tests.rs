use crate::cli::CliOptions;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
  const f1 = await Rsvim.fs.open("README.md");
  f1.close();
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  // Before running
  {
    let contents = lock!(event_loop.contents);
    assert!(
      contents
        .command_line_message()
        .rope()
        .to_string()
        .is_empty()
    );
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let payload = contents.command_line_message().rope().to_string();
    let payload = payload.trim();
    assert!(
      payload
        .contains("\"Rsvim.cmd.echo\" message cannot be undefined or null")
    );
  }

  Ok(())
}
