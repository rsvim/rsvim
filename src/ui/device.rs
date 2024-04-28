use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, execute, terminal};
use std::io::stdout;
// use tracing::debug;

pub async fn init() -> std::io::Result<Device> {
  terminal::enable_raw_mode()?;
  let (cols, rows) = terminal::size()?;
  let device = Device::new(rows as u32, cols as u32);
  // debug!("dvc stat: {:?}", stat);

  execute!(std::io::stdout(), EnableMouseCapture)?;
  execute!(std::io::stdout(), EnableFocusChange)?;

  execute!(
    stdout(),
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBlock,
    cursor::Show,
    cursor::MoveTo(0, 0),
  )?;

  Ok(device)
}

pub async fn shutdown() -> std::io::Result<()> {
  execute!(
    stdout(),
    DisableMouseCapture,
    DisableFocusChange,
    terminal::LeaveAlternateScreen,
  )?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  Ok(())
}

pub struct Device {
  height: u32,
  width: u32,
}

impl Device {
  fn new(height: u32, width: u32) -> Self {
    Device { width, height }
  }
}
