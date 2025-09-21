use crate::cli::CliOptions;
use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
// #[should_panic(
//   expected = "\"Rsvim.cmd.echo\" message parameter cannot be undefined or null"
// )]
async fn test_echo1_should_panic_with_missing_param() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
    Rsvim.cmd.echo();
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let (_tp, path_cfg) = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

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

#[tokio::test]
#[cfg_attr(miri, ignore)]
// #[should_panic(
//   expected = "\"Rsvim.cmd.echo\" message parameter cannot be undefined or null"
// )]
async fn test_echo2_should_panic_with_null_param() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
    Rsvim.cmd.echo(null);
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let (_tp, path_cfg) = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

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
    assert!(payload.contains(
      "\"Rsvim.cmd.echo\" message parameter cannot be undefined or null"
    ));
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_echo3() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
    Rsvim.cmd.echo("");
    Rsvim.cmd.echo("Test echo");
    Rsvim.cmd.echo(123);
    Rsvim.cmd.echo(true);
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let (_tp, path_cfg) = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

  // Before running
  {
    let contents = lock!(event_loop.contents);
    assert_eq!(contents.command_line_message().rope().to_string(), "");
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message().rope().to_string();
    let actual = actual.trim();
    assert!(
      actual.is_empty()
        || actual == "Test echo"
        || actual == "123"
        || actual == "true"
    );
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_echo4() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
  setTimeout(() => {
    Rsvim.cmd.echo("");
    Rsvim.cmd.echo("Test echo");
    Rsvim.cmd.echo(123);
    Rsvim.cmd.echo(true);
  }, 1);
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let (_tp, path_cfg) = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

  // Before running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message().rope().to_string();
    assert!(actual.is_empty());
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message().rope().to_string();
    let actual = actual.trim();
    assert_eq!(actual, "true");
  }

  Ok(())
}
