use crate::cli::CliOptions;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use assert_fs::prelude::FileTouch;
use assert_fs::prelude::FileWriteStr;
use ringbuf::traits::Consumer;
use ringbuf::traits::Observer;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  tmpfile.touch().unwrap();
  info!("tmpfile:{:?}", tmpfile.path());

  let a = format!("{:?}", tmpfile.path());
  let b = format!("{:?}", tmpfile.path().to_string_lossy().to_string());
  info!("a:{a:?}, b:{b:?}, a.contains(b):{}", a.contains(&b));
  assert!(a.contains(&b));

  let src = format!(
    r#"
  const f = await Rsvim.fs.open({:?});
  if (f.isDisposed) {{
    throw new Error("It cannot be closed");
  }}
  f.close();
  if (!f.isDisposed) {{
    throw new Error("It must be closed");
  }}
    "#,
    tmpfile.path()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  tmpfile.touch().unwrap();
  info!("tmpfile:{:?}", tmpfile.path());

  let a = format!("{:?}", tmpfile.path());
  let b = format!("{:?}", tmpfile.path().to_string_lossy().to_string());
  info!("a:{a:?}, b:{b:?}, a.contains(b):{}", a.contains(&b));
  assert!(a.contains(&b));

  let src = format!(
    r#"
  using f = Rsvim.fs.openSync({:?});
    "#,
    tmpfile.path()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close3() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  info!("tmpfile:{:?}", tmpfile);

  let src = format!(
    r#"
  const f = await Rsvim.fs.open({:?}, {{ create: true, write: true }});
  if (f.isDisposed) {{
    throw new Error("It cannot be closed");
  }}
  f.close();
  if (!f.isDisposed) {{
    throw new Error("It must be closed");
  }}
    "#,
    tmpfile.path()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
    assert!(tmpfile.exists());
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close4() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  info!("tmpfile:{:?}", tmpfile);

  let src = format!(
    r#"
  const f = Rsvim.fs.openSync({:?}, {{ create: true, write: true }});
  if (f.isDisposed) {{
    throw new Error("It cannot be closed");
  }}
  f.close();
  if (!f.isDisposed) {{
    throw new Error("It must be closed");
  }}
    "#,
    tmpfile.path()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  event_loop.initialize()?;
  event_loop
    .run_with_mock_events(MockEventReader::new(mocked_events))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
    assert!(tmpfile.exists());
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close_failed1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  info!("tmpfile:{:?}", tmpfile);

  let src = format!(
    r#"
try {{
  const f = await Rsvim.fs.open({:?});
}} catch (e) {{
  Rsvim.cmd.echo(e);
}}
    "#,
    tmpfile.path()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

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
    assert!(actual.contains("Failed to open file"));
    assert!(!tmpfile.exists());
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_open_close_failed2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  info!("tmpfile:{:?}", tmpfile);

  let src = format!(
    r#"
try {{
  const f = Rsvim.fs.openSync({:?});
}} catch (e) {{
  Rsvim.cmd.echo(e);
}}
    "#,
    tmpfile.path()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

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
    assert!(actual.contains("Failed to open file"));
    assert!(!tmpfile.exists());
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_read_write1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  info!("tmpfile:{:?}", tmpfile);
  tmpfile.touch().unwrap();
  tmpfile.write_str("Hello, World!").unwrap();

  let src = format!(
    r#"
  using f = await Rsvim.fs.open({:?});
  const buf = new Uint8Array(100);
  const n = await f.read(buf);
  Rsvim.cmd.echo(`n:${{n}}`);
    "#,
    tmpfile.path()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

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
    let actual = contents
      .command_line_message_history_mut()
      .try_pop()
      .unwrap();
    assert_eq!(actual, "n:13");
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_read_write2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let tmpfile = assert_fs::NamedTempFile::new("README.md").unwrap();
  info!("tmpfile:{:?}", tmpfile);
  tmpfile.touch().unwrap();
  tmpfile.write_str("Hello, World!").unwrap();

  let src = format!(
    r#"
  using f = Rsvim.fs.openSync({:?});
  const buf = new Uint8Array(100);
  const n = await f.read(buf);
  Rsvim.cmd.echo(`n:${{n}}`);
    "#,
    tmpfile.path()
  );
  info!("src:{:?}", src);

  // Prepare $RSVIM_CONFIG/rsvim.js
  let _tp = make_configs(vec![(Path::new("rsvim.js"), &src)]);

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
    let actual = contents
      .command_line_message_history_mut()
      .try_pop()
      .unwrap();
    assert_eq!(actual, "n:13");
  }

  Ok(())
}
