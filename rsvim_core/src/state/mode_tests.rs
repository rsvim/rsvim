use super::mode::*;
use std::str::FromStr;

#[test]
fn test_mode1() {
  assert_eq!(format!("{}", Mode::Normal), "normal");
  assert_eq!(Mode::from_str("normal"), Ok(Mode::Normal));
  assert_eq!(Mode::from_str("n"), Ok(Mode::Normal));

  assert_eq!(format!("{}", Mode::Visual), "visual");
  assert_eq!(Mode::from_str("visual"), Ok(Mode::Visual));
  assert_eq!(Mode::from_str("v"), Ok(Mode::Visual));

  assert_eq!(format!("{}", Mode::Select), "select");
  assert_eq!(Mode::from_str("select"), Ok(Mode::Select));
  assert_eq!(Mode::from_str("s"), Ok(Mode::Select));

  assert_eq!(format!("{}", Mode::OperatorPending), "operator-pending");
  assert_eq!(
    Mode::from_str("operator-pending"),
    Ok(Mode::OperatorPending)
  );
  assert_eq!(Mode::from_str("op-pending"), Ok(Mode::OperatorPending));
  assert_eq!(Mode::from_str("o"), Ok(Mode::OperatorPending));

  assert_eq!(format!("{}", Mode::Insert), "insert");
  assert_eq!(Mode::from_str("insert"), Ok(Mode::Insert));
  assert_eq!(Mode::from_str("i"), Ok(Mode::Insert));

  assert_eq!(format!("{}", Mode::CommandLineEx), "command-line");
  assert_eq!(Mode::from_str("command-line"), Ok(Mode::CommandLineEx));
  assert_eq!(Mode::from_str("cmdline"), Ok(Mode::CommandLineEx));
  assert_eq!(Mode::from_str("c"), Ok(Mode::CommandLineEx));

  assert_eq!(
    format!("{}", Mode::CommandLineSearchForward),
    "command-line-search-forward"
  );
  assert_eq!(
    Mode::from_str("command-line-search-forward"),
    Ok(Mode::CommandLineSearchForward)
  );

  assert_eq!(
    format!("{}", Mode::CommandLineSearchBackward),
    "command-line-search-backward"
  );
  assert_eq!(
    Mode::from_str("command-line-search-backward"),
    Ok(Mode::CommandLineSearchBackward)
  );
}
