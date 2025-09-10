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
  import * as util from "./util.js";
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
      assert!(module_map.counter.pending.is_empty());
      assert!(module_map.pending.is_empty());
      assert!(module_map.counter.failed.is_empty());
      assert_eq!(module_map.counter.resolved.len(), 0);
      info!("module_map.counter:{:?}", module_map.counter);
      assert_eq!(module_map.counter.evaluated.len(), 1);
      assert_eq!(
        module_map.counter.evaluated.get(
          tp.xdg_config_home
            .join("rsvim")
            .join(p1)
            .as_path()
            .to_str()
            .unwrap()
        ),
        Some(&1)
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn no_side_effect2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
  import util from "util";
  util.echo(1);
    "#;

    let p2 = Path::new("util.js");
    let src2: &str = r#"
    function echo(value) {
        Rsvim.cmd.echo(value);
    }
    export default { echo };
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
      assert!(module_map.counter.pending.is_empty());
      assert!(module_map.pending.is_empty());
      assert!(module_map.counter.failed.is_empty());
      assert_eq!(module_map.counter.resolved.len(), 0);
      info!("module_map.counter:{:?}", module_map.counter);
      assert_eq!(module_map.counter.evaluated.len(), 1);
      assert_eq!(
        module_map.counter.evaluated.get(
          tp.xdg_config_home
            .join("rsvim")
            .join(p1)
            .as_path()
            .to_str()
            .unwrap()
        ),
        Some(&1)
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn no_side_effect3() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
  import utils from "utils";
  utils.echo(utils.add(1,2));
    "#;

    let p2 = Path::new("node_modules/utils/lib/echo.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    let p3 = Path::new("node_modules/utils/lib/calc.js");
    let src3: &str = r#"
    export function add(a, b) {
        return a+b;
    }
    "#;

    let p4 = Path::new("node_modules/utils/lib/index.js");
    let src4: &str = r#"
    import {add} from "./calc.js";
    import {echo} from "./echo.js";

    export default {add, echo};
    "#;

    let p5 = Path::new("node_modules/utils/package.json");
    let src5: &str = r#"
{
  "exports": "./lib/index.js"
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_multi_file_configs(
      &tp,
      vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4), (p5, src5)],
    );

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
        Some("3".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );

      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      let module_map = &state.module_map;
      assert!(module_map.counter.pending.is_empty());
      assert!(module_map.pending.is_empty());
      assert!(module_map.counter.failed.is_empty());
      assert_eq!(module_map.counter.resolved.len(), 0);
      info!("module_map.counter:{:?}", module_map.counter);
      assert_eq!(module_map.counter.evaluated.len(), 1);
      assert_eq!(
        module_map.counter.evaluated.get(
          tp.xdg_config_home
            .join("rsvim")
            .join(p1)
            .as_path()
            .to_str()
            .unwrap()
        ),
        Some(&1)
      );
    }

    Ok(())
  }
}

#[cfg(test)]
mod test_dynamic_import {
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
    import("./util.js").then(({echo}) => {
      echo(1);
    }).catch((e) => {
      Rsvim.cdm.echo(`Failed to dynamic import: ${e}`);
    });
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
      info!("module_map.counter:{:?}", module_map.counter());
      assert_eq!(module_map.counter().pending.len(), 1);
      assert!(module_map.pending().borrow().is_empty());
      assert!(module_map.counter().failed.is_empty());
      assert_eq!(module_map.counter().resolved.len(), 1);
      assert_eq!(module_map.counter().evaluated.len(), 2);
      assert_eq!(
        module_map.counter().evaluated.get(
          tp.xdg_config_home
            .join("rsvim")
            .join(p1)
            .as_path()
            .to_str()
            .unwrap()
        ),
        Some(&1)
      );
      assert_eq!(
        module_map.counter().evaluated.get(
          tp.xdg_config_home
            .join("rsvim")
            .join(p2)
            .as_path()
            .to_str()
            .unwrap()
        ),
        Some(&1)
      );
    }

    Ok(())
  }

  // #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn no_side_effect2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
    const util = await import("util");
    util.echo(1);
    "#;

    let p2 = Path::new("util.js");
    let src2: &str = r#"
    function echo(value) {
        Rsvim.cmd.echo(value);
    }
    export default { echo };
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
      assert!(module_map.counter.pending.is_empty());
      assert!(module_map.pending.is_empty());
      assert!(module_map.counter.failed.is_empty());
      assert_eq!(module_map.counter.resolved.len(), 0);
      info!("module_map.counter:{:?}", module_map.counter);
      assert_eq!(module_map.counter.evaluated.len(), 1);
      assert_eq!(
        module_map.counter.evaluated.get(
          tp.xdg_config_home
            .join("rsvim")
            .join(p1)
            .as_path()
            .to_str()
            .unwrap()
        ),
        Some(&1)
      );
    }

    Ok(())
  }

  // #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn no_side_effect3() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
    import("utils")
      .then((utils) => {
        utils.echo(utils.add(3,4));
      }).catch((e) => {
        Rsvim.cmd.echo(`Failed to dynamic import: ${e}`);
      });
    "#;

    let p2 = Path::new("node_modules/utils/lib/echo.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    let p3 = Path::new("node_modules/utils/lib/calc.js");
    let src3: &str = r#"
    export function add(a, b) {
        return a+b;
    }
    "#;

    let p4 = Path::new("node_modules/utils/lib/index.js");
    let src4: &str = r#"
    import {add} from "./calc.js";
    import {echo} from "./echo.js";

    export default {add, echo};
    "#;

    let p5 = Path::new("node_modules/utils/package.json");
    let src5: &str = r#"
{
  "exports": "./lib/index.js"
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_multi_file_configs(
      &tp,
      vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4), (p5, src5)],
    );

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
        Some("3".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );

      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      let module_map = &state.module_map;
      assert!(module_map.counter().pending.is_empty());
      assert!(module_map.pending().borrow().is_empty());
      assert!(module_map.counter().failed.is_empty());
      assert_eq!(module_map.counter().resolved.len(), 0);
      info!(
        "module_map.evaluate_counter:{:?}",
        module_map.counter().evaluated
      );
      assert_eq!(module_map.counter().evaluated.len(), 1);
      assert_eq!(
        module_map.counter().evaluated.get(
          tp.xdg_config_home
            .join("rsvim")
            .join(p1)
            .as_path()
            .to_str()
            .unwrap()
        ),
        Some(&1)
      );
    }

    Ok(())
  }
}
