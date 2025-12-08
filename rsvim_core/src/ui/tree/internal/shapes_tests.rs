use super::shapes::*;
use crate::prelude::*;
// use crate::tests::log::init as test_log_init;
use std::cmp::min;

#[test]
fn convert1() {
  // test_log_init();

  let inputs: Vec<IRect> = vec![
    rect!(0, 0, 3, 5),
    rect!(0, 0, 1, 5),
    rect!(0, 0, 3, 7),
    rect!(0, 0, 0, 0),
    rect!(0, 0, 5, 4),
  ];
  for t in inputs.iter() {
    for p in 0..10 {
      for q in 0..10 {
        let input_actual_parent_shape = rect!(0, 0, p as u16, q as u16);
        let expect =
          rect!(0, 0, min(t.max().x, p) as u16, min(t.max().y, q) as u16);
        let actual =
          convert_relative_to_absolute(t, &input_actual_parent_shape);
        info!("expect:{:?}, actual:{:?}", expect, actual);
        assert_eq!(actual, expect);
      }
    }
  }
}

#[test]
fn convert2() {
  // test_log_init();

  let inputs: Vec<(IRect, U16Rect)> = vec![
    (rect!(0, 0, 3, 5), rect!(0, 0, 7, 8)),
    (rect!(-3, 1, 1, 5), rect!(3, 2, 9, 8)),
    (rect!(3, 9, 6, 10), rect!(1, 1, 2, 2)),
    (rect!(0, 0, 0, 0), rect!(0, 0, 0, 0)),
    (rect!(5, 3, 6, 4), rect!(0, 0, 5, 3)),
  ];
  let expects: Vec<U16Rect> = vec![
    rect!(0, 0, 3, 5),
    rect!(3, 3, 4, 7),
    rect!(2, 2, 2, 2),
    rect!(0, 0, 0, 0),
    rect!(5, 3, 5, 3),
  ];
  for (i, p) in inputs.iter().enumerate() {
    let actual = convert_relative_to_absolute(&p.0, &p.1);
    let expect = expects[i];
    info!(
      "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
      i, p, actual, expect
    );
    assert_eq!(actual, expect);
  }
}

#[test]
fn truncate1() {
  // test_log_init();

  let inputs: Vec<(IRect, U16Rect)> = vec![
    (rect!(0, 0, 7, 8), rect!(0, 0, 10, 10)),
    (rect!(3, 2, 17, 11), rect!(0, 0, 10, 10)),
    (rect!(7, -2, 13, 8), rect!(0, 0, 5, 5)),
    (rect!(-5, 8, 3, 16), rect!(3, 7, 13, 17)),
    (rect!(-5, 17, 1, 21), rect!(10, 15, 18, 23)),
  ];
  let expects: Vec<IRect> = vec![
    rect!(0, 0, 7, 8),
    rect!(0, 1, 10, 10),
    rect!(0, 0, 5, 5),
    rect!(0, 2, 8, 10),
    rect!(0, 4, 6, 8),
  ];
  for (i, p) in inputs.iter().enumerate() {
    let actual = truncate_shape(&p.0, &p.1);
    let expect = expects[i];
    info!(
      "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
      i, p, actual, expect
    );
    assert!(actual == expect);
  }
}
