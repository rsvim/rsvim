use super::attr::*;
use std::str::FromStr;

#[test]
fn test_nargs() {
  assert_eq!(format!("{}", Nargs::Zero), "0");
  assert_eq!(Nargs::from_str("0"), Ok(Nargs::Zero));

  assert_eq!(format!("{}", Nargs::One), "1");
  assert_eq!(Nargs::from_str("1"), Ok(Nargs::One));

  assert_eq!(format!("{}", Nargs::Optional), "?");
  assert_eq!(Nargs::from_str("?"), Ok(Nargs::Optional));

  assert_eq!(format!("{}", Nargs::More), "+");
  assert_eq!(Nargs::from_str("+"), Ok(Nargs::More));

  assert_eq!(format!("{}", Nargs::Any), "*");
  assert_eq!(Nargs::from_str("*"), Ok(Nargs::Any));
}
