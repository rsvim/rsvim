use ringbuf::traits::Consumer;

use crate::cli::CliOptions;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use ringbuf::traits::*;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_timeout1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let src: &str = r#"
  // Set timeout to update global options.
  const timerId = setTimeout(() => {
    Rsvim.opt.wrap = false;
    Rsvim.opt.lineBreak = true;
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

  // Before evaluating javascript configs
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After timeout, it changes to new value
  {
    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert!(!global_local_options.wrap());
    assert!(global_local_options.line_break());
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_timeout2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(20))];
  let src: &str = r#"
  // Set timeout to update global options.
  const timerId = setTimeout(() => {
    Rsvim.opt.wrap = false;
    Rsvim.opt.lineBreak = true;
  }, 100);

  // Cancel the timeout immediately
  clearTimeout(timerId);
"#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let (_tp, path_cfg) = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

  // Before evaluating javascript configs
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // Still remains the same value
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_timeout3() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let src: &str = r#"
  // Set timeout to update global options.
  const timerId = setTimeout((arg1, arg2, arg3) => {
    Rsvim.cmd.echo(arg1);
    Rsvim.cmd.echo(arg2);
    Rsvim.cmd.echo(arg3);
    Rsvim.opt.wrap = false;
    Rsvim.opt.lineBreak = true;
  }, 1, "Hello", "World", true);
"#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let (_tp, path_cfg) = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

  // Before evaluating javascript configs
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After timeout, it changes to new value
  {
    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert!(!global_local_options.wrap());
    assert!(global_local_options.line_break());

    let mut contents = lock!(event_loop.contents);
    assert_eq!(3, contents.command_line_message_history().occupied_len());
    let actual1 = contents.command_line_message_history_mut().try_pop();
    assert!(actual1.is_some());
    let actual1 = actual1.unwrap();
    assert_eq!(actual1, "Hello");

    let actual2 = contents.command_line_message_history_mut().try_pop();
    assert!(actual2.is_some());
    let actual2 = actual2.unwrap();
    assert_eq!(actual2, "World");

    let actual3 = contents.command_line_message_history_mut().try_pop();
    assert!(actual3.is_some());
    let actual3 = actual3.unwrap();
    assert_eq!(actual3, "true");
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_timeout4() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(20))];
  let src: &str = r#"
  // Set timeout to update global options.
  const timerId = setTimeout(() => {
    Rsvim.opt.wrap = false;
    Rsvim.opt.lineBreak = true;
  }, 100);

  // Cancel the timeout immediately
  clearTimeout(timerId);
"#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let (_tp, path_cfg) = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

  // Before evaluating javascript configs
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // Still remains the same value
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  Ok(())
}
