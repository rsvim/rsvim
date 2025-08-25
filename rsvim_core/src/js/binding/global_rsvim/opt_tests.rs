use crate::cli::CliOptions;
use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;

use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_wrap1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];
  let tp = TempPathCfg::create();

  let src: &str = r#"
  const val1 = Rsvim.opt.wrap;
  Rsvim.opt.wrap = false;
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  make_configs(&tp, src);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  // Before running
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert!(!global_local_options.wrap());
  }

  Ok(())
}
