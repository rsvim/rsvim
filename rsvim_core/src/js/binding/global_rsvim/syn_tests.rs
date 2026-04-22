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

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(1000))];
  let src: &str = r#"
  Rsvim.opt.syntaxParserLibPath = ".test-tree-sitter-parsers";
  try {
    await Rsvim.syn.loadTreeSitterParser({ grammarPath: "./tests_and_benchmarks/tree-sitter-c" });
  } catch (e) {
    Rsvim.cmd.echo(e);
  }
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
    let contents = lock!(event_loop.cmdline_text);
    let n = contents.message_history().len();
    assert_eq!(n, 0);
  }

  Ok(())
}
