// use super::module_map::*;
use crate::cli::CliOptions;
use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::constant::TempPathCfg;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use assert_fs::prelude::PathChild;
use ringbuf::traits::*;
use std::path::Path;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn import_meta1() -> IoResult<()> {
  test_log_init();

  let terminal_cols = 10_u16;
  let terminal_rows = 10_u16;
  let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(1000))];
  let tp = TempPathCfg::create();

  let p1 = Path::new("rsvim.js");
  let src1: &str = r#"
import { echoUrl, echoFileName, echoDirName, echoMain } from './utils/a.js';

const url = import.meta.url;
echoUrl(url);

const filename = import.meta.filename;
echoFileName(filename);

const dirname = import.meta.dirname;
echoDirName(dirname);

const isMain = import.meta.main;
echoMain(isMain);

const resolvedModulePath = import.meta.resolve("./utils/a.js");
Rsvim.cmd.echo(resolvedModulePath);
    "#;

  let p2 = Path::new("utils/a.js");
  let src2: &str = r#"
export function echoUrl(value) {
  const url = import.meta.url;
  Rsvim.cmd.echo(`${url}:${value}`);
}

export function echoFileName(value) {
  const filename = import.meta.filename;
  Rsvim.cmd.echo(`${filename}:${value}`);
}

export function echoDirName(value) {
  const dirname = import.meta.dirname;
  Rsvim.cmd.echo(`${dirname}:${value}`);
}

export function echoMain(value) {
  const isMain = import.meta.main;
  Rsvim.cmd.echo(`${isMain}:${value}`);
}
    "#;

  // Prepare $RSVIM_CONFIG
  // |- rsvim.js
  // |- utils/
  //    |- a.js
  //
  make_configs(&tp, vec![(p1, src1), (p2, src2)]);

  let mut event_loop =
    make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

  // Before running
  {
    let contents = lock!(event_loop.contents);
    assert!(contents.command_line_message_history().is_empty());
  }

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let state_rc = event_loop.js_runtime.get_state();
    let state = state_rc.borrow();
    info!("module_map:{:#?}", state.module_map);

    let mut contents = lock!(event_loop.contents);
    assert_eq!(5, contents.command_line_message_history().occupied_len());

    let url = contents.command_line_message_history_mut().try_pop();
    assert!(url.is_some());
    let actual = url.unwrap();
    info!("url:{:?}", actual);
    assert!(
      actual.contains(
        &tp
          .xdg_config_home
          .child("rsvim")
          .child("rsvim.js")
          .to_string_lossy()
          .to_string()
      ) && actual.contains(
        &tp
          .xdg_config_home
          .child("rsvim")
          .child("utils")
          .child("a.js")
          .to_string_lossy()
          .to_string()
      ) && actual
        .match_indices("file://")
        .map(|(i, _)| i)
        .collect::<Vec<_>>()
        .len()
        == 2
    );

    let filename = contents.command_line_message_history_mut().try_pop();
    assert!(filename.is_some());
    let actual = filename.unwrap();
    info!("filename:{:?}", actual);
    assert!(
      actual.contains(
        &tp
          .xdg_config_home
          .child("rsvim")
          .child("rsvim.js")
          .to_string_lossy()
          .to_string()
      ) && actual.contains(
        &tp
          .xdg_config_home
          .child("rsvim")
          .child("utils")
          .child("a.js")
          .to_string_lossy()
          .to_string()
      )
    );

    let dirname = contents.command_line_message_history_mut().try_pop();
    assert!(dirname.is_some());
    let actual = dirname.unwrap();
    info!("dirname:{:?}", actual);
    assert!(
      actual.contains(
        &tp
          .xdg_config_home
          .child("rsvim")
          .to_string_lossy()
          .to_string()
      ) && actual.contains(
        &tp
          .xdg_config_home
          .child("rsvim")
          .child("utils")
          .to_string_lossy()
          .to_string()
      )
    );

    let is_main = contents.command_line_message_history_mut().try_pop();
    assert!(is_main.is_some());
    let actual = is_main.unwrap();
    info!("main:{:?}", actual);
    assert_eq!(actual, "false:true");

    let resolved_module_path =
      contents.command_line_message_history_mut().try_pop();
    assert!(resolved_module_path.is_some());
    let actual = resolved_module_path.unwrap();
    info!("resolve:{:?}", actual);
    assert!(
      actual.contains(
        &tp
          .xdg_config_home
          .child("rsvim")
          .child("utils")
          .child("a.js")
          .to_string_lossy()
          .to_string()
      )
    );
  }

  Ok(())
}
