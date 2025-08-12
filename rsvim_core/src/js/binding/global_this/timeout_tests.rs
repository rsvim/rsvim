use crate::prelude::*;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;

use std::io::Write;
use std::time::Duration;

#[tokio::test]
async fn test_timeout1() -> IoResult<()> {
  test_log_init();

  let tp = TempPathCfg::create();

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(200))];
  let src: &str = r#"
  // Set timeout for 100 milliseconds.
  const timerId = setTimeout(() => {
    Rsvim.opt.wrap = false;
    Rsvim.opt.lineBreak = true;
  }, 100);
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

  let mut event_loop = make_event_loop();

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

  {
    let tree = lock!(event_loop.tree);
    let global_local_options = tree.global_local_options();
    assert!(!global_local_options.wrap());
    assert!(global_local_options.line_break());
  }

  Ok(())
}
