use crate::cli::CliOptions;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use assert_fs::prelude::FileTouch;
use ringbuf::traits::Observer;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  tmpfile.touch().unwrap();

  let src = format!(
    r#"
  const f = await Rsvim.fs.open("{}");
  f.close();
    "#,
    tmpfile.to_string_lossy()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  tmpfile.touch().unwrap();

  let src = format!(
    r#"
  const f = Rsvim.fs.openSync("{}");
  f.close();
    "#,
    tmpfile.to_string_lossy()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
  }

  Ok(())
}
