use crate::cfg::path_cfg::PathConfig;
use crate::cli::CliOptions;
use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::cfg::TempPathConfig;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_exit1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];
  let tp = TempPathConfig::create();

  let src: &str = r#"
  setTimeout(() => {
    Rsvim.rt.exit();
  }, 1);
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  make_configs(&tp, vec![(Path::new("rsvim.js"), src)]);
  let path_cfg = PathConfig::new_with_temp_dirs(&tp);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

  // Before running
  {
    assert_eq!(event_loop.exit_code, 0);
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    assert_eq!(event_loop.exit_code, 0);
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_exit2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];
  let tp = TempPathConfig::create();

  let src: &str = r#"
  setTimeout(() => {
    Rsvim.rt.exit(-1);
  }, 1);
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  make_configs(&tp, vec![(Path::new("rsvim.js"), src)]);
  let path_cfg = PathConfig::new_with_temp_dirs(&tp);

  let mut event_loop = make_event_loop(
    terminal_cols,
    terminal_rows,
    CliOptions::empty(),
    path_cfg,
  );

  // Before running
  {
    assert_eq!(event_loop.exit_code, 0);
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    assert_eq!(event_loop.exit_code, -1);
  }

  Ok(())
}
