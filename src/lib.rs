use std::io::stdout;

use crossterm::{
  style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
  ExecutableCommand,
};

pub fn example() -> std::io::Result<()> {
  stdout()
    .execute(SetForegroundColor(Color::Yellow))?
    .execute(SetBackgroundColor(Color::DarkGrey))?
    .execute(Print("Hello Rsvim!"))?
    .execute(ResetColor)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_example() {
    let actual = example();
    assert_eq!(actual.is_ok(), true);
  }
}
