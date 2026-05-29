use crate::cli::CliOptions;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_new_command1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
  const cmd1 = new Rsvim.proc.Command("ls");
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  // Before running
  {}

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.cmdline_text);
    let n = contents.message_history().len();
    assert_eq!(n, 0);
    //
    // let actual1 = contents.message_history_mut().pop();
    // info!("actual1:{:?}", actual1);
    // assert!(actual1.is_some());
    // let actual1 = actual1.unwrap();
    // assert_eq!(actual1, "c");
    //
    // let actual2 = contents.message_history_mut().pop();
    // info!("actual2:{:?}", actual2);
    // assert!(actual2.is_some());
    // let actual2 = actual2.unwrap();
    // assert_eq!(actual2, "C");
    //
    // let actual3 = contents.message_history_mut().pop();
    // info!("actual3:{:?}", actual3);
    // assert!(actual3.is_some());
    // let actual3 = actual3.unwrap();
    // assert!(actual3.contains("c"));
    // assert!(actual3.contains("h"));
  }

  Ok(())
}
