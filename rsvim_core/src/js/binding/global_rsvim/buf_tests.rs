use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;

use std::time::Duration;

#[cfg(test)]
mod tests_current_buffer {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn undefined1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];
    let tp = TempPathCfg::create();

    let src: &str = r#"
    const bufId = Rsvim.buf.currentBuffer();
    if (bufId != undefined) {
        throw new Error("Current buffer ID is not undefined!");
    }
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(&tp, src);

    let mut event_loop = make_event_loop(terminal_cols, terminal_rows);

    event_loop.initialize()?;
    event_loop.mock_run(MockReader::new(mocked_events)).await?;
    event_loop.shutdown()?;

    // After running
    {
      let contents = lock!(event_loop.contents);
      let payload = contents.command_line_message().rope().to_string();
      let payload = payload.trim();
      assert!(payload.is_empty());
    }

    Ok(())
  }
}
