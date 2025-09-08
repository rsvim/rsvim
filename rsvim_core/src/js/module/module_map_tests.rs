use super::module_map::*;

use crate::cli::CliOptions;
use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;

use ringbuf::traits::*;
use std::path::Path;
use std::time::Duration;

#[cfg(test)]
mod test_static_import {
  use compact_str::ToCompactString;

  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn no_side_effect1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
  import util from "./util.js";
  util.echo(1);
    "#;

    let p2 = Path::new("util.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_multi_file_configs(&tp, vec![(p1, src1), (p2, src2)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let contents = lock!(event_loop.contents);
      assert!(contents.command_line_message_history().is_empty());
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("1".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
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
    let tp = TempPathCfg::create();

    let src: &str = r#"
    Rsvim.cmd.echo(null);
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(&tp, src);

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
    let tp = TempPathCfg::create();

    let src: &str = r#"
    Rsvim.cmd.echo("");
    Rsvim.cmd.echo("Test echo");
    Rsvim.cmd.echo(123);
    Rsvim.cmd.echo(true);
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(&tp, src);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

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
    let tp = TempPathCfg::create();

    let src: &str = r#"
  setTimeout(() => {
    Rsvim.cmd.echo("");
    Rsvim.cmd.echo("Test echo");
    Rsvim.cmd.echo(123);
    Rsvim.cmd.echo(true);
  }, 1);
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(&tp, src);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

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
}
