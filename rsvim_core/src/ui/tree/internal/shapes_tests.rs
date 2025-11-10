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
        let input_actual_parent_shape = U16Rect {
          left: 0,
          top: 0,
          right: p as u16,
          bottom: q as u16,
        };
        let expect = U16Rect {
          left: 0,
          top: 0,
          right: min(t.max().x, p) as u16,
          bottom: min(t.max().y, q) as u16,
        };
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
    (
      IRect {
        left: 0,
        top: 0,
        right: 3,
        bottom: 5,
      },
      U16Rect {
        left: 0,
        top: 0,
        right: 7,
        bottom: 8,
      },
    ),
    (
      IRect {
        left: -3,
        top: 1,
        right: 1,
        bottom: 5,
      },
      U16Rect {
        left: 3,
        top: 2,
        right: 9,
        bottom: 8,
      },
    ),
    (
      IRect {
        left: 3,
        top: 9,
        right: 6,
        bottom: 10,
      },
      U16Rect {
        left: 1,
        top: 1,
        right: 2,
        bottom: 2,
      },
    ),
    (
      IRect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
      },
      U16Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
      },
    ),
    (
      IRect {
        left: 5,
        top: 3,
        right: 6,
        bottom: 4,
      },
      U16Rect {
        left: 0,
        top: 0,
        right: 5,
        bottom: 3,
      },
    ),
  ];
  let expects: Vec<U16Rect> = vec![
    U16Rect {
      left: 0,
      top: 0,
      right: 3,
      bottom: 5,
    },
    U16Rect {
      left: 3,
      top: 3,
      right: 4,
      bottom: 7,
    },
    U16Rect {
      left: 2,
      top: 2,
      right: 2,
      bottom: 2,
    },
    U16Rect {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0,
    },
    U16Rect {
      left: 5,
      top: 3,
      right: 5,
      bottom: 3,
    },
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
        right: 10,
        bottom: 10,
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
        top: -2,
        right: 12,
        bottom: 9,
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
        top: 1,
        right: 12,
        bottom: 9,
      },
      U16Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
      },
    ),
    (
      IRect {
        left: -1,
        top: -1,
        right: 1,
        bottom: 1,
      },
      U16Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
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
      left: 3,
      top: 2,
      right: 10,
      bottom: 10,
    },
    IRect {
      left: 3,
      top: -2,
      right: 12,
      bottom: 8,
    },
    IRect {
      left: 3,
      top: 1,
      right: 3,
      bottom: 1,
    },
    IRect {
      left: -1,
      top: -1,
      right: -1,
      bottom: -1,
    },
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
        right: 12,
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
        right: 10,
        bottom: 10,
      },
    ),
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
