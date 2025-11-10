use super::shapes::*;
use crate::prelude::*;
use crate::rect;
// use crate::tests::log::init as test_log_init;
use std::cmp::min;

#[test]
fn make_actual_shapes1() {
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
        let actual = make_actual_shape(t, &input_actual_parent_shape);
        info!("expect:{:?}, actual:{:?}", expect, actual);
        assert_eq!(actual, expect);
      }
    }
  }
}

#[test]
fn make_actual_shapes2() {
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
    let actual = make_actual_shape(&p.0, &p.1);
    let expect = expects[i];
    info!(
      "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
      i, p, actual, expect
    );
    assert_eq!(actual, expect);
  }
}

#[test]
fn bound_size1() {
  // test_log_init();

  let inputs: Vec<(IRect, U16Rect)> = vec![
    (rect!(0, 0, 7, 8), rect!(0, 0, 10, 10)),
    (rect!(3, 2, 10, 10), rect!(0, 0, 10, 10)),
    (rect!(3, -2, 12, 9), rect!(0, 0, 10, 10)),
    (rect!(3, 1, 12, 9), rect!(0, 0, 0, 0)),
    (rect!(-1, -1, 1, 1), rect!(0, 0, 0, 0)),
  ];
  let expects: Vec<IRect> = vec![
    rect!(0, 0, 7, 8),
    rect!(3, 2, 10, 10),
    rect!(3, -2, 12, 8),
    rect!(3, 1, 3, 1),
    rect!(-1, -1, -1, -1),
  ];
  for (i, p) in inputs.iter().enumerate() {
    let actual = bound_size(&p.0, &p.1);
    let expect = expects[i];
    info!(
      "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
      i, p, actual, expect
    );
    assert!(actual == expect);
  }
}

#[test]
fn bound_position1() {
  // test_log_init();

  let inputs: Vec<(IRect, U16Rect)> = vec![
    (rect!(0, 0, 7, 8), rect!(0, 0, 10, 10)),
    (rect!(3, 2, 12, 11), rect!(0, 0, 10, 10)),
    (rect!(7, -2, 13, 8), rect!(0, 0, 10, 10)),
    (
      IRect {
        left: -8,
        top: 8,
        right: -3,
        bottom: 16,
      },
      U16Rect {
        left: 3,
        top: 7,
        right: 8,
        bottom: 15,
      },
    ),
    (
      IRect {
        left: -5,
        top: 19,
        right: -3,
        bottom: 21,
      },
      U16Rect {
        left: 10,
        top: 15,
        right: 15,
        bottom: 20,
      },
    ),
  ];
  let expects: Vec<IRect> = vec![
    IRect {
      left: 0,
      top: 0,
      right: 7,
      bottom: 8,
    },
    IRect {
      left: 1,
      top: 1,
      right: 10,
      bottom: 10,
    },
    IRect {
      left: 4,
      top: 0,
      right: 10,
      bottom: 10,
    },
    IRect {
      left: 0,
      top: 0,
      right: 5,
      bottom: 8,
    },
    IRect {
      left: 0,
      top: 3,
      right: 2,
      bottom: 5,
    },
  ];
  for (i, p) in inputs.iter().enumerate() {
    let actual = bound_position(&p.0, &p.1);
    let expect = expects[i];
    info!(
      "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
      i, p, actual, expect
    );
    assert!(actual == expect);
  }
}

#[test]
fn bound_shape1() {
  // test_log_init();

  let inputs: Vec<(IRect, U16Rect)> = vec![
    (
      IRect {
        left: 0,
        top: 0,
        right: 7,
        bottom: 8,
      },
      U16Rect {
        left: 0,
        top: 0,
        right: 10,
        bottom: 10,
      },
    ),
    (
      IRect {
        left: 3,
        top: 2,
        right: 17,
        bottom: 11,
      },
      U16Rect {
        left: 0,
        top: 0,
        right: 10,
        bottom: 10,
      },
    ),
    (
      IRect {
        left: 7,
        top: -2,
        right: 13,
        bottom: 8,
      },
      U16Rect {
        left: 0,
        top: 0,
        right: 5,
        bottom: 5,
      },
    ),
    (
      IRect {
        left: -5,
        top: 8,
        right: 3,
        bottom: 16,
      },
      U16Rect {
        left: 3,
        top: 7,
        right: 13,
        bottom: 17,
      },
    ),
    (
      IRect {
        left: -5,
        top: 17,
        right: 1,
        bottom: 21,
      },
      U16Rect {
        left: 10,
        top: 15,
        right: 18,
        bottom: 23,
      },
    ),
  ];
  let expects: Vec<IRect> = vec![
    IRect {
      left: 0,
      top: 0,
      right: 7,
      bottom: 8,
    },
    IRect {
      left: 0,
      top: 1,
      right: 10,
      bottom: 10,
    },
    IRect {
      left: 0,
      top: 0,
      right: 5,
      bottom: 5,
    },
    IRect {
      left: 0,
      top: 2,
      right: 8,
      bottom: 10,
    },
    IRect {
      left: 0,
      top: 4,
      right: 6,
      bottom: 8,
    },
  ];
  for (i, p) in inputs.iter().enumerate() {
    let actual = bound_shape(&p.0, &p.1);
    let expect = expects[i];
    info!(
      "i:{:?}, input:{:?}, actual:{:?}, expect:{:?}",
      i, p, actual, expect
    );
    assert!(actual == expect);
  }
}
