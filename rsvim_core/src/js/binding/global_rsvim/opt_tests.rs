use crate::buf::opt::*;
use crate::cli::CliOptions;
use crate::prelude::*;
use crate::results::IoResult;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use crate::ui::widget::window::opt::*;
use std::time::Duration;

#[cfg(test)]
mod tests_wrap {

  use super::*;
  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_wrap1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  const val1 = Rsvim.opt.wrap;
  Rsvim.opt.wrap = false;
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let tree = lock!(event_loop.tree);
      let global_local_options = tree.global_local_options();
      assert_eq!(global_local_options.wrap(), WRAP);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let tree = lock!(event_loop.tree);
      let global_local_options = tree.global_local_options();
      assert!(!global_local_options.wrap());
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests_tab_stop {

  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn success1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  const v1 = Rsvim.opt.tabStop;
  Rsvim.opt.tabStop = 4;
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.tab_stop(), TAB_STOP);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.tab_stop(), 4);

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      assert!(actual.trim().is_empty());
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn failed1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  Rsvim.opt.tabStop = -1;
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.tab_stop(), TAB_STOP);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.tab_stop(), TAB_STOP);

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      let actual = actual.trim();
      info!("actual:{actual}");
      let expect = r####""Rsvim.opt.tabStop" parameter must be between [1,255], but found"####;
      assert!(actual.contains(expect));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests_file_encoding {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn success1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  const v2 = Rsvim.opt.fileEncoding;
  Rsvim.opt.fileEncoding = "utf-8";
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.tab_stop(), TAB_STOP);
      assert_eq!(global_local_options.file_encoding(), FILE_ENCODING);
      assert_eq!(global_local_options.file_format(), FILE_FORMAT);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      use crate::buf::opt::FileEncodingOption;

      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(
        global_local_options.file_encoding(),
        FileEncodingOption::Utf8,
      );

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      assert!(actual.trim().is_empty());
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn failed1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  Rsvim.opt.fileEncoding = "utf-16";
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.file_encoding(), FILE_ENCODING);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.file_encoding(), FILE_ENCODING);

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      let actual = actual.trim();
      info!("actual:{actual}");
      let expect = r####""Rsvim.opt.fileEncoding" parameter is invalid"####;
      assert!(actual.contains(expect));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests_file_format {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn success1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  const v3 = Rsvim.opt.fileFormat;
  Rsvim.opt.fileFormat = "mac";
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.tab_stop(), TAB_STOP);
      assert_eq!(global_local_options.file_encoding(), FILE_ENCODING);
      assert_eq!(global_local_options.file_format(), FILE_FORMAT);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      use crate::buf::opt::FileFormatOption;

      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.file_format(), FileFormatOption::Mac);

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      assert!(actual.trim().is_empty());
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn failed1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  Rsvim.opt.fileFormat = "CRLF";
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.file_format(), FILE_FORMAT);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.file_format(), FILE_FORMAT);

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      let actual = actual.trim();
      info!("actual:{actual}");
      let expect = r####""Rsvim.opt.fileFormat" parameter is invalid"####;
      assert!(actual.contains(expect));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests_expand_tab {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn success1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  const value = Rsvim.opt.expandTab;
  Rsvim.opt.expandTab = true;
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.expand_tab(), EXPAND_TAB);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert!(global_local_options.expand_tab());

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      assert!(actual.trim().is_empty());
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn failed1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  Rsvim.opt.expandTab = null;
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.expand_tab(), EXPAND_TAB);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.expand_tab(), EXPAND_TAB);

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      let actual = actual.trim();
      info!("actual:{actual}");
      let expect =
        r####""Rsvim.opt.expandTab" parameter must be a boolean, but found"####;
      assert!(actual.contains(expect));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests_shift_width {
  use super::*;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn success1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  const value = Rsvim.opt.shiftWidth;
  Rsvim.opt.shiftWidth = 4;
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.shift_width(), SHIFT_WIDTH);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.shift_width(), 4);

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      assert!(actual.trim().is_empty());
    }

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn failed1() -> IoResult<()> {
    test_log_init();

    let terminal_cols = 10_u16;
    let terminal_rows = 10_u16;
    let mocked_ops = vec![MockOperation::SleepFor(Duration::from_millis(30))];

    let src: &str = r#"
  Rsvim.opt.shiftWidth = 10000;
    "#;

    // Prepare $RSVIM_CONFIG/rsvim.js
    let _tp = make_configs(vec![(Path::new("rsvim.js"), src)]);

    let mut event_loop =
      make_event_loop(terminal_cols, terminal_rows, CliOptions::empty());

    // Before running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.shift_width(), SHIFT_WIDTH);
    }

    event_loop.initialize()?;
    event_loop
      .run_with_mock_operations(MockOperationReader::new(mocked_ops))
      .await?;
    event_loop.shutdown()?;

    // After running
    {
      let buffers = lock!(event_loop.buffers);
      let global_local_options = buffers.global_local_options();
      assert_eq!(global_local_options.shift_width(), SHIFT_WIDTH);

      let contents = lock!(event_loop.contents);
      let actual = contents.command_line_message().rope().to_string();
      let actual = actual.trim();
      info!("actual:{actual}");
      let expect = r####""Rsvim.opt.shiftWidth" parameter must be between [1,255], but found"####;
      assert!(actual.contains(expect));
    }

    Ok(())
  }
}
