use crate::ui::canvas::Canvas;
use crate::ui::rect::{AbsPos, Size};
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, execute, queue, terminal};
use std::io::{stdout, Result as IoResult};

struct Screen {
  size: Size,
  pos: AbsPos,
}

impl Canvas for Screen {
  async fn new() -> IoResult<Screen> {
    terminal::enable_raw_mode()?;
    let (cols, rows) = terminal::size()?;
    let screen = Screen {
      size: Size::new(rows as u32, cols as u32),
      pos: AbsPos::new(0, 0),
    };

    let mut out = std::io::stdout();

    queue!(out, EnableMouseCapture)?;
    queue!(out, EnableFocusChange)?;

    queue!(
      out,
      terminal::EnterAlternateScreen,
      terminal::Clear(terminal::ClearType::All),
      cursor::SetCursorStyle::BlinkingBlock,
      cursor::Show,
      cursor::MoveTo(0, 0),
    )?;

    out.flush()?;

    Ok(screen)
  }

  async fn shutdown(&self) -> IoResult<()> {
    let mut out = std::io::stdout();
    queue!(
      out,
      DisableMouseCapture,
      DisableFocusChange,
      terminal::LeaveAlternateScreen,
    )?;

    out.flush()?;

    if terminal::is_raw_mode_enabled()? {
      terminal::disable_raw_mode()?;
    }

    Ok(())
  }

  fn height(&self) -> u32 {
    self.size.height
  }

  fn width(&self) -> u32 {
    self.size.width
  }

  fn x(&self) -> u32 {
    self.pos.x
  }

  fn y(&self) -> u32 {
    self.pos.y
  }
}
