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
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- util.js
  //
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
    make_configs(&tp, vec![(p1, src1), (p2, src2)]);

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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

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
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- util.js
  //
  async fn no_side_effect2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
  import util from "util.js";
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
    make_configs(&tp, vec![(p1, src1), (p2, src2)]);

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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

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
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- node_modules/
  //    |- utils/
  //       |- package.json
  //       |- lib/
  //          |- index.js
  //          |- echo.js
  //          |- calc.js
  //
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
    make_configs(
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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("3".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- utils/
  //    |- a.js
  //    |- b.js
  //    |- c.js
  //    |- d.js
  //
  async fn no_side_effect5() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
import { echoA } from './utils/a.js';

echoA(5);
    "#;

    let p2 = Path::new("utils/a.js");
    let src2: &str = r#"
import { echoB } from './b.js';

export function echoA(value) {
  echoB(`A:${value}`);
}
    "#;

    let p3 = Path::new("utils/b.js");
    let src3: &str = r#"
import { echoC } from './c.js';

export function echoB(value) {
  echoC(`B:${value}`);
}
    "#;

    let p4 = Path::new("utils/c.js");
    let src4: &str = r#"
import { echoD } from './d.js';

export function echoC(value) {
  echoD(`C:${value}`);
}
    "#;

    let p5 = Path::new("utils/d.js");
    let src5: &str = r#"
export function echoD(value) {
  Rsvim.cmd.echo(`D:${value}`);
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(
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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("D:C:B:A:5".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- node_modules/
  //    |- utils/
  //       |- package.json
  //       |- lib/
  //          |- index.js
  //          |- echo.js
  //          |- calc.js
  //
  async fn side_effect1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
  import utils from "utils";
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

    echo(add(4,5));
    export default {};
    "#;

    let p5 = Path::new("node_modules/utils/package.json");
    let src5: &str = r#"
{
  "exports": "./lib/index.js"
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(
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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("9".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
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
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- util.js
  //
  async fn no_side_effect1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
    import("./util.js").then(({echo}) => {
      echo(1);
    }).catch((e) => {
      Rsvim.cdm.echo(`Failed to dynamic import: ${e}`);
    }).finally(() => {
      Rsvim.rt.exit(0);
    });
    "#;

    let p2 = Path::new("util.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(&tp, vec![(p1, src1), (p2, src2)]);

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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

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
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- util.js
  //
  async fn no_side_effect2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
try {
  const util = await import("util.js");
  util.echo(1);
} catch (e) {
  Rsvim.cmd.echo(`Failed to dynamic import util: ${e}`);
}
Rsvim.rt.exit(0);
    "#;

    let p2 = Path::new("util.js");
    let src2: &str = r#"
    function echo(value) {
        Rsvim.cmd.echo(value);
    }
    export { echo };
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(&tp, vec![(p1, src1), (p2, src2)]);

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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      info!(
        "command_line_message_history occupied_len:{}, vacant_len: {}",
        contents.command_line_message_history().occupied_len(),
        contents.command_line_message_history().vacant_len()
      );
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
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- node_modules/
  //    |- utils/
  //       |- package.json
  //       |- lib/
  //          |- index.js
  //          |- echo.js
  //          |- calc.js
  //
  async fn no_side_effect3() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
    import("utils")
      .then((utils) => {
        utils.echo(utils.add(3,4));
      }).catch((e) => {
        Rsvim.cmd.echo(`Failed to dynamic import: ${e}`);
      }).finally(() => {
        Rsvim.rt.exit(0);
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

    export {add, echo};
    "#;

    let p5 = Path::new("node_modules/utils/package.json");
    let src5: &str = r#"
{
  "exports": "./lib/index.js"
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(
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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("7".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- utils/
  //    |- index.js
  //    |- echo.js
  //    |- calc.js
  //
  async fn no_side_effect4() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
    import "./utils";
    "#;

    let p2 = Path::new("utils/echo.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    let p3 = Path::new("utils/calc.js");
    let src3: &str = r#"
    export function add(a, b) {
        return a+b;
    }
    "#;

    let p4 = Path::new("utils/index.js");
    let src4: &str = r#"
try {
  const { add } = await import('./calc.js');
  const { echo } = await import('./echo.js');
  echo(add(4, 5));
} catch (e) {
  Rsvim.cmd.echo(`Failed to dynamic import: ${e}`);
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(&tp, vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4)]);

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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("9".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- utils/
  //    |- a.js
  //    |- b.js
  //    |- c.js
  //    |- d.js
  //
  async fn no_side_effect5() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
try {
  const { echoA } = await import('./utils/a.js');
  echoA(5);
} catch (e) {
  console.log(`Failed to dynamic import:${e}`);
}
    "#;

    let p2 = Path::new("utils/a.js");
    let src2: &str = r#"
import { echoB } from './b.js';

export function echoA(value) {
  echoB(`A:${value}`);
}
    "#;

    let p3 = Path::new("utils/b.js");
    let src3: &str = r#"
import { echoC } from './c.js';

export function echoB(value) {
  echoC(`B:${value}`);
}
    "#;

    let p4 = Path::new("utils/c.js");
    let src4: &str = r#"
import { echoD } from './d.js';

export function echoC(value) {
  echoD(`C:${value}`);
}
    "#;

    let p5 = Path::new("utils/d.js");
    let src5: &str = r#"
export function echoD(value) {
  Rsvim.cmd.echo(`D:${value}`);
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(
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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("D:C:B:A:5".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- node_modules/
  //    |- utils/
  //       |- package.json
  //       |- lib/
  //          |- index.js
  //          |- echo.js
  //          |- calc.js
  //
  async fn side_effect1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(300))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
    import utils from "utils";
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
try {
  const {add} = await import("./calc.js");
  const {echo} = await import("./echo.js");
  echo(add(4,5));
} catch(e) {
  Rsvim.cmd.echo(`Failed to dynamic import calc/echo: ${e}`);
}

export default {};
    "#;

    let pkg5 = Path::new("node_modules/utils/package.json");
    let pkg_src5: &str = r#"
{
  "exports": "./lib/index.js"
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(
      &tp,
      vec![
        (p1, src1),
        (p2, src2),
        (p3, src3),
        (p4, src4),
        (pkg5, pkg_src5),
      ],
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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("9".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  // Config structure:
  //
  // ${RSVIM_CONFIG_HOME}
  // |- rsvim.js
  // |- node_modules/
  //    |- utils/
  //       |- package.json
  //       |- lib/
  //          |- index.js
  //          |- echo.js
  //          |- calc.js
  //
  async fn side_effect2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(300))];
    let tp = TempPathCfg::create();

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
    import "utils";
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
try {
  const {add} = await import("./calc.js");
  const {echo} = await import("./echo.js");
  echo(add(4,5));
} catch(e) {
  Rsvim.cmd.echo(`Failed to dynamic import calc/echo: ${e}`);
}

export default {};
    "#;

    let pkg5 = Path::new("node_modules/utils/package.json");
    let pkg_src5: &str = r#"
{
  "exports": "./lib/index.js"
}
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    make_configs(
      &tp,
      vec![
        (p1, src1),
        (p2, src2),
        (p3, src3),
        (p4, src4),
        (pkg5, pkg_src5),
      ],
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
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      assert_eq!(
        Some("9".to_compact_string()),
        contents.command_line_message_history_mut().try_pop()
      );
    }

    Ok(())
  }
}
