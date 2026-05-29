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
  if (typeof cmd1 !== "object") {{
    throw new Error("cmd1 is not object");
  }}
  if (!Array.isArray(cmd1.args) || cmd1.args.length > 0) {{
    throw new Error("cmd1.args must be empty array");
  }}
  if (cmd1.clearEnv !== false) {{
    throw new Error("cmd1.clearEnv must be false");
  }}
  if (cmd1.cwd !== null && cmd1.cwd !== undefined) {{
    throw new Error("cmd1.clearEnv must be null or undefined");
  }}
  if (cmd1.detached !== false) {{
    throw new Error("cmd1.detached must be false");
  }}
  if (typeof cmd1.env !== "object" || Object.keys(cmd1.env).length > 0) {{
    throw new Error("cmd1.env must be empty object");
  }}
  if (cmd1.stdin !== "null") {{
    throw new Error("cmd1.stdin is not 'null'");
  }}
  if (cmd1.stdout !== "piped") {{
    throw new Error("cmd1.stdout is not 'piped'");
  }}
  if (cmd1.stderr !== "piped") {{
    throw new Error("cmd1.stderr is not 'piped'");
  }}
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
  }

  Ok(())
}
