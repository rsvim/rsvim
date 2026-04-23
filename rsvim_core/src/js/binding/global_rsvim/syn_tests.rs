use crate::cli::CliOptions;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_load1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(10000))];

  // The runtime path is "./rsvim_core", so we need to prepend ".." for it.
  let src: &str = r#"
  Rsvim.opt.syntaxParserLibPath = ".test-tree-sitter-parsers";
  try {
    const parsers = await Rsvim.syn.loadParser({ grammarPath: "../tests_and_benchmarks/tree-sitter-c" });
    Rsvim.cmd.echo(parsers);
  } catch (e) {
    Rsvim.cmd.echo(e);
  }
  Rsvim.rt.exit(0);
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

  // After running
  {
    let mut contents = lock!(event_loop.cmdline_text);
    let n = contents.message_history().len();
    assert_eq!(n, 1);
    let actual = contents.message_history_mut().pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert_eq!(actual, "c");
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_load_sync1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(10000))];

  // The runtime path is "./rsvim_core", so we need to prepend ".." for it.
  let src: &str = r#"
  Rsvim.opt.syntaxParserLibPath = ".test-tree-sitter-parsers";
  try {
    const parsers = Rsvim.syn.loadParserSync({ grammarPath: "../tests_and_benchmarks/tree-sitter-python" });
    Rsvim.cmd.echo(parsers);
  } catch (e) {
    Rsvim.cmd.echo(e);
  }
  Rsvim.rt.exit(0);
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

  // After running
  {
    let mut contents = lock!(event_loop.cmdline_text);
    let n = contents.message_history().len();
    assert_eq!(n, 1);
    let actual = contents.message_history_mut().pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert_eq!(actual, "python");
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_list1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(10000))];

  // The runtime path is "./rsvim_core", so we need to prepend ".." for it.
  let src: &str = r#"
  Rsvim.opt.syntaxParserLibPath = ".test-tree-sitter-parsers";
  try {
    const parsers = Rsvim.syn.loadParserSync({ grammarPath: "../tests_and_benchmarks/tree-sitter-python" });
    Rsvim.cmd.echo(parsers);
    const allParsers = Rsvim.syn.listParsers();
    Rsvim.cmd.echo(allParsers);
  } catch (e) {
    Rsvim.cmd.echo(e);
  }
  Rsvim.rt.exit(0);
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

  // After running
  {
    let mut contents = lock!(event_loop.cmdline_text);
    let n = contents.message_history().len();
    assert_eq!(n, 2);
    let actual1 = contents.message_history_mut().pop();
    info!("actual1:{:?}", actual1);
    assert!(actual1.is_some());
    let actual1 = actual1.unwrap();
    assert_eq!(actual1, "python");

    let actual2 = contents.message_history_mut().pop();
    info!("actual2:{:?}", actual2);
    assert!(actual2.is_some());
    let actual2 = actual2.unwrap();
    assert_eq!(actual2, "python");
  }

  Ok(())
}
