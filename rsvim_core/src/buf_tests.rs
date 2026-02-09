use super::buf::*;
use crate::cli::CliOptions;
use crate::cli::SpecialCliOptions;
use crate::prelude::*;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::GotoInsertModeVariant;
use crate::state::ops::Operation;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use compact_str::ToCompactString;
use regex::Regex;
use std::time::Duration;

#[test]
fn next_buffer_id1() {
  assert!(BufferId::next() > 0);
}

#[cfg(test)]
mod tests_undo {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn undefined1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
    const buf = Rsvim.buf.current();
    if (buf !== undefined) {
        throw new Error("Current buffer ID is not undefined!");
    }
    const bufs = Rsvim.buf.list();
    if (!Array.isArray(bufs)) {
        throw new Error("Buffers is not an array!");
    }
    if (bufs.length > 0) {
        throw new Error("Buffers list is not empty!");
    }
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let contents = lock!(event_loop.contents);
      let payload = contents.cmdline_message().rope().to_string();
      info!("After payload:{payload:?}");
      let payload = payload.trim();
      assert!(payload.is_empty());
    }

    Ok(())
  }
}
