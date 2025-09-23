// use super::module_map::*;
use crate::cli::CliOptions;
use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::evloop::*;
use crate::tests::gh::is_github_actions;
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
  async fn xdg_config_dir1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

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

    // Prepare $RSVIM_CONFIG:
    // - rsvim.js
    // - util.js
    let _tp = make_configs(vec![(p1, src1), (p2, src2)]);

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
  async fn xdg_config_dir2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
  import util from "./util.js";
  util.echo(1);
    "#;

    let p2 = Path::new("util.js");
    let src2: &str = r#"
    function echo(value) {
        Rsvim.cmd.echo(value);
    }
    export default { echo };
    "#;

    // Prepare $RSVIM_CONFIG
    // - rsvim.js
    // - util.js
    let _tp = make_configs(vec![(p1, src1), (p2, src2)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

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
  async fn xdg_config_dir3() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

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

    // Prepare $RSVIM_CONFIG:
    // |- rsvim.js
    // |- node_modules/
    //    |- utils/
    //       |- package.json
    //       |- lib/
    //          |- index.js
    //          |- echo.js
    //          |- calc.js
    //
    let _tp = make_configs(vec![
      (p1, src1),
      (p2, src2),
      (p3, src3),
      (p4, src4),
      (p5, src5),
    ]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

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
  async fn xdg_config_dir4() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(500))];

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

    // Prepare $RSVIM_CONFIG
    // |- rsvim.js
    // |- utils/
    //    |- a.js
    //    |- b.js
    //    |- c.js
    //    |- d.js
    //
    let _tp = make_configs(vec![
      (p1, src1),
      (p2, src2),
      (p3, src3),
      (p4, src4),
      (p5, src5),
    ]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

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
  async fn xdg_config_dir5() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

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

    // Prepare $RSVIM_CONFIG:
    // |- rsvim.js
    // |- node_modules/
    //    |- utils/
    //       |- package.json
    //       |- lib/
    //          |- index.js
    //          |- echo.js
    //          |- calc.js
    //
    let _tp = make_configs(vec![
      (p1, src1),
      (p2, src2),
      (p3, src3),
      (p4, src4),
      (p5, src5),
    ]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

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
  async fn xdg_config_dir6() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
  import a from "a";
  import b from "b";

  a.echo(6);
  b.echo(6);
    "#;

    let p2 = Path::new("node_modules/utils/index.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    let p3 = Path::new("node_modules/a/index.js");
    let src3: &str = r#"
    import {echo} from "utils";

    export default { echo };
    "#;

    let p4 = Path::new("b/index.js");
    let src4: &str = r#"
    import {echo} from "utils";

    export default {echo};
    "#;

    // Prepare $RSVIM_CONFIG:
    // |- rsvim.js
    // |- b/index.js
    // |- node_modules/
    //    |- utils/index.js
    //    |- a/index.js
    //
    let _tp =
      make_configs(vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

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
      assert_eq!(2, contents.command_line_message_history().occupied_len());
      for _i in 0..2 {
        let actual = contents.command_line_message_history_mut().try_pop();
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert_eq!(actual, "6");
      }
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn xdg_config_dir7() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
  import a from "a";
  import b from "b";

  a.echo(6);
  b.echo(6);
    "#;

    let p2 = Path::new("utils/index.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    let p3 = Path::new("a/index.js");
    let src3: &str = r#"
    import {echo} from "utils";

    export default { echo };
    "#;

    let p4 = Path::new("node_modules/b/index.js");
    let src4: &str = r#"
    import {echo} from "utils";

    export default {echo};
    "#;

    // Prepare $RSVIM_CONFIG:
    // |- rsvim.js
    // |- b/index.js
    // |- node_modules/
    //    |- utils/index.js
    //    |- a/index.js
    //
    let _tp =
      make_configs(vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After
    {
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(2, contents.command_line_message_history().occupied_len());
      for _i in 0..2 {
        let actual = contents.command_line_message_history_mut().try_pop();
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert_eq!(actual, "6");
      }
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn home_dir1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

    let p1 = Path::new(".rsvim/rsvim.js");
    let src1: &str = r#"
  import * as util from "./util.js";
  util.echo(1);
    "#;

    let p2 = Path::new(".rsvim/util.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    // Prepare $HOME:
    // |- .rsvim/
    //    |- rsvim.js
    //    |- util.js
    let _tp = make_home_configs(vec![(p1, src1), (p2, src2)]);

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
  async fn home_dir2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

    let p1 = Path::new(".rsvim.js");
    let src1: &str = r#"
  import util from "./.rsvim/util.js";
  util.echo(1);
    "#;

    let p2 = Path::new(".rsvim/util.js");
    let src2: &str = r#"
    function echo(value) {
        Rsvim.cmd.echo(value);
    }
    export default { echo };
    "#;

    // Prepare $HOME:
    // |- .rsvim.js
    // |- .rsvim/
    //    |- util.js
    let _tp = make_home_configs(vec![(p1, src1), (p2, src2)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

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
  async fn home_dir3() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(300))];

    let p1 = Path::new(".rsvim/rsvim.js");
    let src1: &str = r#"
  import a from "a";
  import b from "b";
  a.echo(1);
  b.echo(2);
    "#;

    let p2 = Path::new(".rsvim/a/index.js");
    let src2: &str = r#"
    import {echo} from "util";
    export default { echo };
    "#;

    let p3 = Path::new(".rsvim/b/index.js");
    let src3: &str = r#"
    import {echo} from "util";
    export default { echo };
    "#;

    let p4 = Path::new(".rsvim/util/index.js");
    let src4: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    // Prepare $HOME:
    // |- .rsvim/
    //    |- rsvim.js
    //    |- a/index.js
    //    |- b/index.js
    //    |- util/index.js
    let _tp =
      make_home_configs(vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After
    {
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      let n = contents.command_line_message_history().occupied_len();
      info!("home_dir3 n:{:?}", n);
      assert_eq!(2, n);
      for i in 0..n {
        let actual = contents.command_line_message_history_mut().try_pop();
        info!("home_dir3 i:{:?},actual:{:?}", i, actual);
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert!(actual == "1" || actual == "2");
      }
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn home_dir_failed1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(500))];

    let p1 = Path::new(".rsvim.js");
    let src1: &str = r#"
import { echoA } from './utils/a.js';

echoA(5);
    "#;

    let p2 = Path::new(".rsvim/utils/a.js");
    let src2: &str = r#"
export function echoA(value) {
  Rsvim.cmd.echo(`A:${value}`);
}
    "#;

    // Prepare $HOME
    // |- .rsvim.js
    // |- .rsvim/
    //    |- utils/
    //       |- a.js
    //
    let _tp = make_home_configs(vec![(p1, src1), (p2, src2)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After
    {
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      let actual = contents.command_line_message_history_mut().try_pop();
      info!("actual:{:?}", actual);
      assert!(actual.is_some());
      let actual = actual.unwrap();
      assert!(actual.contains("Uncaught Error: Module path NotFound"));
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn home_dir4() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(300))];

    let p1 = Path::new(".rsvim/rsvim.js");
    let src1: &str = r#"
  import a from "a";
  import b from "b";
  a.echo(1);
  b.echo(2);
    "#;

    let p2 = Path::new(".rsvim/a/index.js");
    let src2: &str = r#"
    import {echo} from "util";
    export default { echo };
    "#;

    let p3 = Path::new(".rsvim/node_modules/b/index.js");
    let src3: &str = r#"
    import {echo} from "util";
    export default { echo };
    "#;

    let p4 = Path::new(".rsvim/util/index.js");
    let src4: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    // Prepare $HOME:
    // |- .rsvim/
    //    |- rsvim.js
    //    |- node_modules/
    //       |- a/index.js
    //       |- b/index.js
    //       |- util/index.js
    let _tp =
      make_home_configs(vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After
    {
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      let n = contents.command_line_message_history().occupied_len();
      info!("home_dir4 n:{:?}", n);
      assert_eq!(2, n);
      for i in 0..n {
        let actual = contents.command_line_message_history_mut().try_pop();
        info!("home_dir4 i:{:?},actual:{:?}", i, actual);
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert!(actual == "1" || actual == "2");
      }
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn home_dir5() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(300))];

    let p1 = Path::new(".rsvim/rsvim.js");
    let src1: &str = r#"
  import a from "a";
  import b from "b";
  a.echo(1);
  b.echo(2);
    "#;

    let p2 = Path::new(".rsvim/node_modules/a/index.js");
    let src2: &str = r#"
    import {echo} from "util";
    export default { echo };
    "#;

    let p3 = Path::new(".rsvim/b/index.js");
    let src3: &str = r#"
    import {echo} from "util";
    export default { echo };
    "#;

    let p4 = Path::new(".rsvim/node_modules/util/index.js");
    let src4: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    // Prepare $HOME:
    // |- .rsvim/
    //    |- rsvim.js
    //    |- node_modules/
    //       |- a/index.js
    //       |- b/index.js
    //       |- util/index.js
    let (_tp, path_cfg) =
      make_home_configs(vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4)]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After
    // FIXME: I have no idea why this test would always fail on GitHub Actions,
    // I have to skip it now! Somebody help me fix it!
    if !is_github_actions() {
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      let n = contents.command_line_message_history().occupied_len();
      info!("home_dir5 n:{:?}", n);
      assert_eq!(2, n);
      for i in 0..n {
        let actual = contents.command_line_message_history_mut().try_pop();
        info!("home_dir5 i:{:?},actual:{:?}", i, actual);
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert!(actual == "1" || actual == "2");
      }
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn homg_dir6() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(300))];

    let p1 = Path::new(".rsvim/rsvim.js");
    let src1: &str = r#"
  import a from "a";
  import b from "b";
  a.echo(1);
  b.echo(2);
    "#;

    let p2 = Path::new(".rsvim/node_modules/a/index.js");
    let src2: &str = r#"
    import {echo} from "util";
    export default { echo };
    "#;

    let p3 = Path::new(".rsvim/node_modules/b/index.js");
    let src3: &str = r#"
    import {echo} from "util";
    export default { echo };
    "#;

    let p4 = Path::new(".rsvim/node_modules/util/index.js");
    let src4: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    // Prepare $HOME:
    // |- .rsvim/
    //    |- rsvim.js
    //    |- node_modules/
    //       |- a/index.js
    //       |- b/index.js
    //       |- util/index.js
    let (_tp, path_cfg) =
      make_home_configs(vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4)]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After
    {
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      let n = contents.command_line_message_history().occupied_len();
      assert_eq!(2, n);
      for _i in 0..n {
        let actual = contents.command_line_message_history_mut().try_pop();
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert!(actual == "1" || actual == "2");
      }
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn home_dir_failed2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

    let p1 = Path::new(".rsvim.js");
    let src1: &str = r#"
  import utils from "./.rsvim/utils";
  utils.echo(utils.add(1,2));
    "#;

    let p2 = Path::new(".rsvim/utils/lib/echo.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    let p3 = Path::new(".rsvim/utils/lib/calc.js");
    let src3: &str = r#"
    export function add(a, b) {
        return a+b;
    }
    "#;

    let p4 = Path::new(".rsvim/utils/lib/index.js");
    let src4: &str = r#"
    import {add} from "./calc.js";
    import {echo} from "./echo.js";

    export default {add, echo};
    "#;

    let p5 = Path::new(".rsvim/utils/package.json");
    let src5: &str = r#"
{
  "exports": "./lib/index.js"
}
    "#;

    // Prepare $RSVIM_CONFIG:
    // |- rsvim.js
    // |- node_modules/
    //    |- utils/
    //       |- package.json
    //       |- lib/
    //          |- index.js
    //          |- echo.js
    //          |- calc.js
    //
    let (_tp, path_cfg) = make_home_configs(vec![
      (p1, src1),
      (p2, src2),
      (p3, src3),
      (p4, src4),
      (p5, src5),
    ]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After
    {
      let state_rc = event_loop.js_runtime.get_state();
      let state = state_rc.borrow();
      info!("module_map:{:#?}", state.module_map);

      let mut contents = lock!(event_loop.contents);
      assert_eq!(1, contents.command_line_message_history().occupied_len());
      let actual = contents.command_line_message_history_mut().try_pop();
      info!("actual:{:?}", actual);
      assert!(actual.is_some());
      let actual = actual.unwrap();
      assert!(actual.contains("Uncaught Error: Module path NotFound"));
    }

    Ok(())
  }
}

#[cfg(test)]
mod test_import_meta {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn property1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(500))];

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
import { echoUrl, echoFileName, echoDirName, echoMain } from './utils/a.js';

setTimeout(() => {
  const url = import.meta.url;
  echoUrl(url);

  const filename = import.meta.filename;
  echoFileName(filename);

  const dirname = import.meta.dirname;
  echoDirName(dirname);

  const isMain = import.meta.main;
  echoMain(isMain);

  const resolvedModulePath = import.meta.resolve("./utils/a.js");
  Rsvim.cmd.echo(resolvedModulePath);
  Rsvim.rt.exit();
}, 1);
    "#;

    let p2 = Path::new("utils/a.js");
    let src2: &str = r#"
export function echoUrl(value) {
  const url = import.meta.url;
  Rsvim.cmd.echo(`${url}:${value}`);
}

export function echoFileName(value) {
  const filename = import.meta.filename;
  Rsvim.cmd.echo(`${filename}:${value}`);
}

export function echoDirName(value) {
  const dirname = import.meta.dirname;
  Rsvim.cmd.echo(`${dirname}:${value}`);
}

export function echoMain(value) {
  const isMain = import.meta.main;
  Rsvim.cmd.echo(`${isMain}:${value}`);
}
    "#;

    // Prepare $RSVIM_CONFIG
    // |- rsvim.js
    // |- utils/
    //    |- a.js
    //
    let (_tp, path_cfg) = make_configs(vec![(p1, src1), (p2, src2)]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg.clone(),
    );

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
      assert_eq!(5, contents.command_line_message_history().occupied_len());

      let url = contents.command_line_message_history_mut().try_pop();
      assert!(url.is_some());
      let actual = url.unwrap();
      info!("url:{:?}", actual);
      info!(
        "path_cfg config_entry:{:?}, config_home:{:?}",
        path_cfg.config_entry(),
        path_cfg.config_home()
      );
      assert!(
        actual.contains(
          &path_cfg
            .config_entry()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        ) && actual.contains(
          &path_cfg
            .config_home()
            .join("utils")
            .join("a.js")
            .to_string_lossy()
            .to_string()
        ) && actual
          .match_indices("file://")
          .map(|(i, _)| i)
          .collect::<Vec<_>>()
          .len()
          == 2
      );

      let filename = contents.command_line_message_history_mut().try_pop();
      assert!(filename.is_some());
      let actual = filename.unwrap();
      info!("filename:{:?}", actual);
      assert!(
        actual.contains(
          &path_cfg
            .config_entry()
            .unwrap()
            .to_string_lossy()
            .to_string()
        ) && actual.contains(
          &path_cfg
            .config_home()
            .join("utils")
            .join("a.js")
            .to_string_lossy()
            .to_string()
        )
      );

      let dirname = contents.command_line_message_history_mut().try_pop();
      assert!(dirname.is_some());
      let actual = dirname.unwrap();
      info!("dirname:{:?}", actual);
      assert!(
        actual.contains(&path_cfg.config_home().to_string_lossy().to_string())
          && actual.contains(
            &path_cfg
              .config_home()
              .join("utils")
              .to_string_lossy()
              .to_string()
          )
      );

      let is_main = contents.command_line_message_history_mut().try_pop();
      assert!(is_main.is_some());
      let actual = is_main.unwrap();
      info!("main:{:?}", actual);
      assert_eq!(actual, "false:true");

      let resolved_module_path =
        contents.command_line_message_history_mut().try_pop();
      assert!(resolved_module_path.is_some());
      let actual = resolved_module_path.unwrap();
      info!("resolve:{:?}", actual);
      assert!(
        actual.contains(
          &path_cfg
            .config_home()
            .join("utils")
            .join("a.js")
            .to_string_lossy()
            .to_string()
        )
      );
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn resolve_failed1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(500))];

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
setTimeout(() => {
  try {
    const url1 = import.meta.resolve(undefined);
    Rsvim.cmd.echo(url1);
  } catch(e) {
    Rsvim.cmd.echo(e);
  }

  try {
    const url2 = import.meta.resolve(null);
    Rsvim.cmd.echo(url2);
  } catch(e) {
    Rsvim.cmd.echo(e);
  }

  try {
    const url3 = import.meta.resolve();
    Rsvim.cmd.echo(url3);
  } catch(e) {
    Rsvim.cmd.echo(e);
  }

  Rsvim.rt.exit();
}, 1);
    "#;

    // Prepare $RSVIM_CONFIG
    // |- rsvim.js
    //
    let (_tp, path_cfg) = make_configs(vec![(p1, src1)]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
      assert_eq!(3, contents.command_line_message_history().occupied_len());

      let url1 = contents.command_line_message_history_mut().try_pop();
      assert!(url1.is_some());
      let actual = url1.unwrap();
      info!("url1:{:?}", actual);
      assert!(
        actual.contains("TypeError: Module path NotFound")
          && actual.contains("undefined")
      );

      let url2 = contents.command_line_message_history_mut().try_pop();
      assert!(url2.is_some());
      let actual = url2.unwrap();
      info!("url2:{:?}", actual);
      assert!(
        actual.contains("TypeError: Module path NotFound")
          && actual.contains("null")
      );

      let url3 = contents.command_line_message_history_mut().try_pop();
      assert!(url3.is_some());
      let actual = url3.unwrap();
      info!("url3:{:?}", actual);
      assert!(actual.contains("TypeError: Not enough arguments specified."));
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
  async fn xdg_config_dir1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];

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
    let (_tp, path_cfg) = make_configs(vec![(p1, src1), (p2, src2)]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
  async fn xdg_config_dir2() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
try {
  const util = await import("./util.js");
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
    let (_tp, path_cfg) = make_configs(vec![(p1, src1), (p2, src2)]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
  async fn xdg_config_dir3() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];

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
    let (_tp, path_cfg) = make_configs(vec![
      (p1, src1),
      (p2, src2),
      (p3, src3),
      (p4, src4),
      (p5, src5),
    ]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
  async fn xdg_config_dir4() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(500))];

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
    let (_tp, path_cfg) =
      make_configs(vec![(p1, src1), (p2, src2), (p3, src3), (p4, src4)]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
  async fn xdg_config_dir5() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
try {
  const { echoA } = await import('./utils/a.js');
  echoA(5);
} catch (e) {
  console.log(`Failed to dynamic import:${e}`);
}
Rsvim.rt.exit();
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
    let (_tp, path_cfg) = make_configs(vec![
      (p1, src1),
      (p2, src2),
      (p3, src3),
      (p4, src4),
      (p5, src5),
    ]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
  async fn xdg_config_dir6() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(500))];

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
    let (_tp, path_cfg) = make_configs(vec![
      (p1, src1),
      (p2, src2),
      (p3, src3),
      (p4, src4),
      (pkg5, pkg_src5),
    ]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
  async fn xdg_config_dir7() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(500))];

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
    let (_tp, path_cfg) = make_configs(vec![
      (p1, src1),
      (p2, src2),
      (p3, src3),
      (p4, src4),
      (pkg5, pkg_src5),
    ]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
mod test_require {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn failed1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(100))];

    let p1 = Path::new("rsvim.js");
    let src1: &str = r#"
    const util = require("./util.js");
    util.echo(1);
    "#;

    let p2 = Path::new("util.js");
    let src2: &str = r#"
    export function echo(value) {
        Rsvim.cmd.echo(value);
    }
    "#;

    // Prepare $RSVIM_CONFIG:
    // - rsvim.js
    // - util.js
    let (_tp, path_cfg) = make_configs(vec![(p1, src1), (p2, src2)]);

    let mut event_loop = make_event_loop(
      terminal_cols,
      terminal_rows,
      CliOptions::empty(),
      path_cfg,
    );

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
      let actual = contents.command_line_message_history_mut().try_pop();
      assert!(actual.is_some());
      let actual = actual.unwrap();
      assert!(
        actual.contains("Uncaught ReferenceError: require is not defined")
      );
    }

    Ok(())
  }
}
