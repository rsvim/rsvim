use super::attr::*;
use std::str::FromStr;

#[test]
fn test_nargs() {
  assert_eq!(format!("{}", Nargs::Zero), "0");
  assert_eq!(Nargs::from_str("0"), Ok(Nargs::Zero));

  assert_eq!(
    format!("{}", Nargs::Count(std::num::NonZeroU8::new(1).unwrap())),
    "1"
  );
  assert_eq!(
    Nargs::from_str("1"),
    Ok(Nargs::Count(std::num::NonZeroU8::new(1).unwrap()))
  );
}
