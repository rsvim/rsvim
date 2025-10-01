use super::converter::*;
use crate::cli::CliOptions;
use crate::tests::evloop::*;

#[test]
fn test_integer1() {
  let ev = make_event_loop(10, 10, CliOptions::empty());
}
