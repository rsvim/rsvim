// use super::command::*;
use crate::cli::CliOptions;
use crate::cli::CliSpecialOptions;
use crate::prelude::*;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use assert_fs::prelude::FileTouch;
use assert_fs::prelude::FileWriteStr;
use compact_str::ToCompactString;
use ringbuf::traits::*;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_js_echo1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("js Rsvim.cmd.echo(1);".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
    MockOperation::SleepFor(Duration::from_millis(50)),
  ];

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), "")]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert_eq!(actual, "1");
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_js_throw1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
  throw new Error(1);
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);
    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert!(actual.contains("Uncaught Error: 1"));
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_js_invalid1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(30))];

  let src: &str = r#"
  asdf
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);
    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert!(actual.contains("Uncaught ReferenceError: asdf is not defined"));
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_buf_write1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("w".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
    MockOperation::SleepFor(Duration::from_millis(50)),
  ];

  let src: &str = r#"
function write(ctx) {
  const bufId = Rsvim.buf.current();
  try {
    const n = Rsvim.buf.writeSync(bufId);
    Rsvim.cmd.echo(`Buffer ${bufId} have been saved, ${n} bytes written.`);
  } catch (e) {
    Rsvim.cmd.echo(`Failed to write buffer ${bufId}: ${e}`);
  }
}

Rsvim.cmd.create("write", write, {}, {alias: "w"});
  "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let buf_file = tp.xdg_data_home.join("test.txt");
  let cli_opts =
    CliOptions::new(CliSpecialOptions::empty(), vec![buf_file], true);

  let mut event_loop = make_event_loop(terminal_cols, terminal_rows, cli_opts);

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert!(actual.starts_with("Buffer") && actual.ends_with("bytes written."));
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_buf_write2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("w".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
    MockOperation::SleepFor(Duration::from_millis(50)),
  ];

  let src: &str = r#"
function write(ctx) {
  const bufId = ctx.currentBufferId;
  try {
    const n = Rsvim.buf.writeSync(bufId);
    Rsvim.cmd.echo(`Buffer ${bufId} have been saved, ${n} bytes written.`);
  } catch (e) {
    Rsvim.cmd.echo(`Failed to write buffer ${bufId}: ${e}`);
  }
}

Rsvim.cmd.create("write", write, {}, {alias: "w"});
  "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

  let buf_file = tp.xdg_data_home.join("test.txt");
  let cli_opts =
    CliOptions::new(CliSpecialOptions::empty(), vec![buf_file], true);

  let mut event_loop = make_event_loop(terminal_cols, terminal_rows, cli_opts);

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert!(actual.starts_with("Buffer") && actual.ends_with("bytes written."));
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_buf_write_failed1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("w".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
    MockOperation::SleepFor(Duration::from_millis(50)),
  ];

  let src: &str = r#"
function write() {
  const bufId = Rsvim.buf.current();
  try {
    const n = Rsvim.buf.writeSync(bufId);
    Rsvim.cmd.echo(`Buffer ${bufId} have been saved, ${n} bytes written.`);
  } catch (e) {
    Rsvim.cmd.echo(`Failed to write buffer ${bufId}: ${e}`);
  }
}

Rsvim.cmd.create("write", write, {}, {alias: "w"});
  "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);
  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert!(actual.starts_with("Failed to write buffer"));
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_async_command() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("msg".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
    MockOperation::SleepFor(Duration::from_millis(50)),
  ];

  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  tmpfile.touch().unwrap();
  tmpfile.write_str("Hello, World").unwrap();
  info!("tmpfile:{:?}", tmpfile.path());

  let src = format!(
    r#"
async function msg() {{
  try {{
    const f = await Rsvim.fs.open({:?});
    const buf = new Uint8Array(100);
    const n = await f.read(buf);
    Rsvim.cmd.echo(`n:${{n}}`);
  }} catch (e) {{
    Rsvim.cmd.echo(`Failed:${{e}}`);
  }}
}}

Rsvim.cmd.create("msg", msg);
  "#,
    tmpfile.path()
  );

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

  let cli_opts = CliOptions::empty();
  let mut event_loop = make_event_loop(terminal_cols, terminal_rows, cli_opts);

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert_eq!(actual.unwrap(), "n:12");
  }

  Ok(())
}
