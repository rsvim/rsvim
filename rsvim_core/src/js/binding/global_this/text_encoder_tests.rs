use crate::cli::CliOptions;
use crate::prelude::*;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use ringbuf::traits::*;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_encode1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let src: &str = r#"
  const encoder = new TextEncoder();
  const s1 = "This is some data";
  const s2 = "你好，世界！";
  const bytes1 = encoder.encode(s1);
  const bytes2 = encoder.encode(s2);

  if (bytes1.byteLength !== 17) {
    Rsvim.cmd.echo(`bytes1 failed, bytesLen:${bytes1.byteLength}`);
  }

  if (bytes2.byteLength !== 18 ) {
    Rsvim.cmd.echo(`bytes2 failed, bytesLen:${bytes2.byteLength}`);
  }
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

  // After
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_encode2() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let src: &str = r#"
  const encoder = new TextEncoder();
  const s1 = "This is some data";
  const s2 = "你好，世界！";
  const buf1 = new Uint8Array(s1.length * 4);
  const buf2 = new Uint8Array(s2.length * 4);
  const res1 = encoder.encodeInto(s1, buf1);
  const res2 = encoder.encodeInto(s2, buf2);

  if (res1.read !== s1.length || res1.written !== 17) {
    Rsvim.cmd.echo(`buf1 failed, res1.read:${res1.read}, res1.written:${res1.written}, s1.length:${s1.length}`);
  }

  if (res2.read !== s2.length || res2.written !== 18) {
    Rsvim.cmd.echo(`buf2 failed, res2.read:${res2.read}, res2.written:${res2.written}, s2.length:${s2.length}`);
  }
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

  // After
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
  }

  Ok(())
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn test_decode1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;

  let mocked_events = vec![MockEvent::SleepFor(Duration::from_millis(50))];
  let src: &str = r#"
  const s1 = "This is some data";
  const s2 = "你好，世界！";
  const encoder = new TextEncoder();
  const bytes1 = encoder.encode(s1);
  const bytes2 = encoder.encode(s2);

  const decoder = new TextDecoder();
  const s3 = decoder.decode(bytes1);
  const s4 = decoder.decode(bytes2);

  if (s1 !== s3) {
    Rsvim.cmd.echo("bytes1 failed");
  }
  if (s2 !== s4) {
    Rsvim.cmd.echo("bytes2 failed");
  }
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

  // After
  {
    let contents = lock!(event_loop.contents);
    let actual = contents.command_line_message_history().is_empty();
    assert!(actual);
  }

  Ok(())
}
