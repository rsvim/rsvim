// use super::module_map::*;

use crate::cli::CliOptions;
use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;

use compact_str::ToCompactString;
use ringbuf::traits::*;
use std::path::Path;
use std::time::Duration;

#[cfg(test)]
mod test_static_import {
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

      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      let module_map = &state.module_map;
      assert_eq!(module_map.pending_counter().len(), 1);
      assert_eq!(module_map.pending_counter().get("./util.js"), Some(&1));
      assert_eq!(module_map.evaluate_counter().len(), 1);
      assert_eq!(
        module_map.evaluate_counter().get("rsvim/rsvim.js"),
        Some(&1)
      );
    }

    Ok(())
  }
}
