use crate::prelude::*;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;

use std::io::Write;
use std::time::Duration;

#[tokio::test]
async fn test_timeout1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let tp = TempPathCfg::create();

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let src: &str = r#"
  // Set timeout to update global options.
  const timerId = setTimeout(() => {
    Rsvim.opt.wrap = false;
    Rsvim.opt.lineBreak = true;
  }, 10);
"#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  {
    std::fs::create_dir_all(tp.xdg_config_home.join("rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tp.xdg_config_home.join("rsvim").join("rsvim.js"))
        .unwrap();
    config_entry.write_all(src.as_bytes()).unwrap();
    config_entry.flush().unwrap();
  }

  let mut event_loop = make_event_loop(terminal_cols, terminal_rows);

  // Before evaluating javascript configs
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  event_loop.initialize()?;
  event_loop.mock_run(MockReader::new(mocked_events)).await?;
  event_loop.shutdown()?;

  // After timeout, it changes to new value
  {
    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert!(!global_local_options.wrap());
    assert!(global_local_options.line_break());
  }

  Ok(())
}

#[tokio::test]
async fn test_timeout2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let tp = TempPathCfg::create();

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let src: &str = r#"
  // Set timeout to update global options.
  const timerId = setTimeout(() => {
    Rsvim.opt.wrap = false;
    Rsvim.opt.lineBreak = true;
  }, 100);

  // Cancel the timeout immediately
  clearTimeout(timerId);
"#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  {
    std::fs::create_dir_all(tp.xdg_config_home.join("rsvim")).unwrap();
    let mut config_entry =
      std::fs::File::create(tp.xdg_config_home.join("rsvim").join("rsvim.js"))
        .unwrap();
    config_entry.write_all(src.as_bytes()).unwrap();
    config_entry.flush().unwrap();
  }

  let mut event_loop = make_event_loop(terminal_cols, terminal_rows);

  // Before evaluating javascript configs
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  event_loop.initialize()?;
  event_loop.mock_run(MockReader::new(mocked_events)).await?;
  event_loop.shutdown()?;

  // Still remains the same value
  {
    use crate::defaults;

    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert_eq!(global_local_options.wrap(), defaults::win::WRAP);
    assert_eq!(global_local_options.line_break(), defaults::win::LINE_BREAK);
  }

  Ok(())
}
