use crate::results::IoResult;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::make_event_loop;
use crate::tests::log::init as test_log_init;
use std::io::Write;

#[tokio::test]
#[should_panic(
  expected = "\"Rsvim.cmd.echo\" message parameter cannot be undefined or null"
)]
async fn test_echo1_should_panic_with_missing_param() {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let tp = TempPathCfg::create();

  let src: &str = r#"
    Rsvim.cmd.echo();
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

  event_loop.initialize().unwrap();
}

#[tokio::test]
#[should_panic(
  expected = "\"Rsvim.cmd.echo\" message parameter cannot be undefined or null"
)]
async fn test_echo2_should_panic_with_null_param() {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let tp = TempPathCfg::create();

  let src: &str = r#"
    Rsvim.cmd.echo(null);
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

  event_loop.initialize().unwrap();
}

#[tokio::test]
async fn test_echo3() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let tp = TempPathCfg::create();

  let src: &str = r#"
    Rsvim.cmd.echo("");
    Rsvim.cmd.echo("Test echo");
    Rsvim.cmd.echo(123);
    Rsvim.cmd.echo(true);
    "#;

  // Prepare $RSVIM_CONFIG/rsvim.js
  {
    std::fs::create_dir_all(tp.xdg_config_home.join("rsvim"))?;
    let mut config_entry =
      std::fs::File::create(tp.xdg_config_home.join("rsvim").join("rsvim.js"))?;
    config_entry.write_all(src.as_bytes())?;
    config_entry.flush()?;
  }

  let mut event_loop = make_event_loop(terminal_cols, terminal_rows);

  event_loop.initialize()?;
  event_loop.shutdown()?;

  Ok(())
}
