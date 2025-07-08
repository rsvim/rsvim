use super::ascii::*;

use ascii::AsciiChar;

#[test]
fn display() {
  for i in 0_u32..32_u32 {
    let ac = AsciiChar::from_ascii(i).unwrap();
    let fmt = AsciiControlCodeFormatter::from(ac);
    println!("{i}:{fmt}");
  }
}
