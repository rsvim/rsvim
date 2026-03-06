use super::cell::*;
use compact_str::CompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;

#[test]
fn default1() {
  let c = Cell::default();
  assert_eq!(c.symbol(), "");
  assert_eq!(c.fg(), &Color::Reset);
  assert_eq!(c.bg(), &Color::Reset);
  assert_eq!(c.attr(), &Attributes::default());
}

#[test]
fn new1() {
  let c1 = Cell::new(
    CompactString::new(" "),
    Color::Reset,
    Color::Reset,
    Attributes::default(),
  );
  let c2 = Cell::default();
  assert_eq!(c1.symbol(), " ");
  assert_eq!(c2.symbol(), "");
  assert_eq!(c1.fg(), &Color::Reset);
  assert_eq!(c1.fg(), c2.fg());
  assert_eq!(c1.bg(), &Color::Reset);
  assert_eq!(c1.bg(), c2.bg());
  assert_eq!(c1.attr(), &Attributes::default());
  assert_eq!(c1.attr(), c2.attr());
}

#[test]
fn from1() {
  let expects = ['a', 'b', 'c', 'd', 'e', 'F', 'G', 'H', 'I'];
  for (i, input) in expects.iter().enumerate() {
    let c: Cell = (*input).into();
    let s = c.symbol().as_str();
    let cs: Vec<char> = s.chars().collect();
    let expect = expects[i];
    assert!(s.len() == 1);
    assert!(cs.len() == 1);
    assert!(cs[0] == expect);
  }
}
