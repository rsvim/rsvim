use super::buf::*;
use crate::cli::CliOptions;
use crate::cli::SpecialCliOptions;
use crate::prelude::*;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::GotoInsertModeVariant;
use crate::state::ops::Operation;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use compact_str::ToCompactString;
use regex::Regex;
use std::time::Duration;

#[test]
fn next_buffer_id1() {
  assert!(BufferId::next() > 0);
}
