use super::timeout::*;

use crate::constant::path_config::*;
use crate::js::loader::ModuleLoader;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use crate::{evloop, prelude::*};

use assert_fs::prelude::*;
use std::fs;
use std::path::Path;
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

  let mut event_loop = make_event_loop();
  event_loop.initialize()?;
  event_loop.mock_run(MockReader::new(mocked_events)).await?;
  event_loop.shutdown()?;

  Ok(())
}
