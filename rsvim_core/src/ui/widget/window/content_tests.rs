#[allow(unused_imports)]
use super::content::*;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::opt::FileFormatOption;
use crate::prelude::*;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::buf::make_empty_buffer;
use crate::tests::log::init as test_log_init;
use crate::tests::viewport::assert_canvas;
use crate::tests::viewport::make_canvas;
use crate::tests::viewport::make_viewport;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;

struct Arguments<'a> {
  pub terminal_size: U16Size,
  pub buffer_opts: BufferOptions,
  pub window_opts: WindowOptions,
  pub buffer_lines: Vec<&'a str>,
  pub expect_canvas: Vec<Vec<&'a str>>,
  pub viewport_start: Vec<(usize, usize)>,
}

fn run_lines(args: Arguments) {
  test_log_init();
  let buf = make_buffer_from_lines(
    args.terminal_size,
    args.buffer_opts,
    args.buffer_lines,
  );

  assert_eq!(args.expect_canvas.len(), args.viewport_start.len());

  for (i, expect) in args.expect_canvas.iter().enumerate() {
    let viewport_start = args.viewport_start[i];
    let viewport = make_viewport(
      args.terminal_size,
      args.window_opts,
      buf.clone(),
      viewport_start.0,
      viewport_start.1,
    );
    let actual =
      make_canvas(args.terminal_size, args.window_opts, buf.clone(), viewport);
    assert_canvas(&actual, expect);
  }
}

fn run_empty(args: Arguments) {
  test_log_init();
  let buf = make_empty_buffer(args.terminal_size, args.buffer_opts);

  assert_eq!(args.expect_canvas.len(), args.viewport_start.len());

  for (i, expect) in args.expect_canvas.iter().enumerate() {
    let viewport_start = args.viewport_start[i];
    let viewport = make_viewport(
      args.terminal_size,
      args.window_opts,
      buf.clone(),
      viewport_start.0,
      viewport_start.1,
    );
    let actual =
      make_canvas(args.terminal_size, args.window_opts, buf.clone(), viewport);
    assert_canvas(&actual, expect);
  }
}

#[cfg(test)]
mod tests_nowrap {
  use super::*;

  #[test]
  fn new1() {
    let buffer_lines: Vec<&str> = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "Hello, RSV",
      "This is a ",
      "But still ",
      "  1. When ",
      "  2. When ",
      "     * The",
      "     * The",
      "          ",
      "          ",
      "          ",
    ];

    run_lines(Arguments {
      terminal_size: size!(10, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new2() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "Hello, RSVIM!                      ",
      "This is a quite simple and small te",
      "But still it contains several thing",
      "  1. When the line is small enough ",
      "  2. When the line is too long to b",
      "     * The extra parts are been tru",
    ];

    run_lines(Arguments {
      terminal_size: size!(35, 6),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new3() {
    let buffer_lines = vec![
      "Hello,  R\tS\tV\tI\tM!\n",
      "这是一个非常简单而且非常短的测试例子，只包含几行文本内容。\n",
      "But still\tit\tcontains\tseveral things we want to test:\n",
      "  第一，当一行文本内容足够短，以至于能够被完全的放入一个窗口中时，then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "Hello,  R        S        V<<<<<<",
      "这是一个非常简单而且非常短的测试<",
      "But still        it        contai",
      "  第一，当一行文本内容足够短，以<",
      "  2. When the line is too long to",
      "     * The extra parts are been t",
      "     * The extra parts are split ",
      "                                 ",
      "                                 ",
      "                                 ",
    ];

    run_lines(Arguments {
      terminal_size: size!(33, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new4() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "Hello, RSVIM!                  ",
      "This is a quite simple and smal",
      "But still it contains several t",
      "  1. When the line is small eno",
      "  2. When the line is too long ",
      "     * The extra parts are been",
      "     * The extra parts are spli",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
    ];

    run_lines(Arguments {
      terminal_size: size!(31, 20),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new5() {
    let expect = vec![
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
    ];

    run_empty(Arguments {
      terminal_size: size!(31, 20),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines: vec![],
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new6() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "Hello, RSVIM!",
      "This is a qui",
      "But still it ",
      "  1. When the",
      "  2. When the",
      "     * The ex",
      "     * The ex",
      "             ",
      "             ",
      "             ",
    ];

    run_lines(Arguments {
      terminal_size: size!(13, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn update1() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect1 = vec![
      "Hello, RSVIM!        ",
      "This is a quite simpl",
      "But still it contains",
      "  1. When the line is",
      "  2. When the line is",
      "     * The extra part",
      "     * The extra part",
      "                     ",
      "                     ",
      "                     ",
    ];
    let expect2 = vec![
      "  2. When the line is",
      "     * The extra part",
      "     * The extra part",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
    ];

    run_lines(Arguments {
      terminal_size: size!(21, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect1, expect2],
      viewport_start: vec![(0, 0), (4, 0)],
    });
  }
}

#[cfg(test)]
mod tests_nowrap_eol {
  use super::*;

  #[test]
  fn new1_crlf_win() {
    let buffer_lines = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple and small test lines.\r\n",
      "But still it contains several things we want to test:\r\n",
    ];
    let expect = vec![
      "Hello, RSVIM!                  ",
      "This is a quite simple and smal",
      "But still it contains several t",
      "                               ",
      "                               ",
    ];

    run_lines(Arguments {
      terminal_size: size!(31, 5),
      buffer_opts: BufferOptionsBuilder::default()
        .file_format(FileFormatOption::Dos)
        .build()
        .unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new1_cr_mac() {
    let buffer_lines = vec![
      "Hello, RSVIM!\r",
      "This is a quite simple and small test lines.\r",
      "But still it contains several things we want to test:\r",
    ];
    let expect = vec![
      "Hello, RSVIM!                  ",
      "This is a quite simple and smal",
      "But still it contains several t",
      "                               ",
      "                               ",
    ];

    run_lines(Arguments {
      terminal_size: size!(31, 5),
      buffer_opts: BufferOptionsBuilder::default()
        .file_format(FileFormatOption::Mac)
        .build()
        .unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new2_crlf_win() {
    let buffer_lines = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple and small test lines.\r\n",
      "But still it contains several things we want to test:\r\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r\n",
    ];
    let expect = vec![
      "Hello, RSVIM!                      ",
      "This is a quite simple and small te",
      "But still it contains several thing",
      "  1. When the line is small enough ",
      "  2. When the line is too long to b",
      "     * The extra parts are been tru",
    ];

    run_lines(Arguments {
      terminal_size: size!(35, 6),
      buffer_opts: BufferOptionsBuilder::default()
        .file_format(FileFormatOption::Dos)
        .build()
        .unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new2_cr_mac() {
    let buffer_lines = vec![
      "Hello, RSVIM!\r",
      "This is a quite simple and small test lines.\r",
      "But still it contains several things we want to test:\r",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r",
    ];
    let expect = vec![
      "Hello, RSVIM!                      ",
      "This is a quite simple and small te",
      "But still it contains several thing",
      "  1. When the line is small enough ",
      "  2. When the line is too long to b",
      "     * The extra parts are been tru",
    ];

    run_lines(Arguments {
      terminal_size: size!(35, 6),
      buffer_opts: BufferOptionsBuilder::default()
        .file_format(FileFormatOption::Mac)
        .build()
        .unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }
}

#[cfg(test)]
mod tests_nowrap_startcol {
  use super::*;

  #[test]
  fn new1() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "o, RSVIM! ",
      " is a quit",
      "still it c",
      " When the ",
      " When the ",
      " * The ext",
      " * The ext",
      "          ",
      "          ",
      "          ",
    ];

    run_lines(Arguments {
      terminal_size: size!(10, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 4)],
    });
  }

  #[test]
  fn new2() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "                                   ",
      "                                   ",
      "a row of the window content widget,",
      " of the window content widget, ther",
      "and word-wrap options are not set. ",
      "r line-wrap or word-wrap options ar",
    ];

    run_lines(Arguments {
      terminal_size: size!(35, 6),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(1, 60)],
    });
  }

  #[test]
  fn new3() {
    let buffer_lines = vec![
      "Hello,  R\tS\tV\tI\tM!\n",
      "这是一个非常简单而且非常短的测试例子，只包含几行文本内容。\n",
      "But still\tit\tcontains\tseveral things we want to test:\n",
      "  第一，当一行文本内容足够短，以至于能够被完全的放入一个窗口中时，then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "S        V        I        M!    ",
      ">且非常短的测试例子，只包含几行文",
      "it        contains        several",
      ">内容足够短，以至于能够被完全的放",
      "e is too long to be completely pu",
      "parts are been truncated if both ",
      "parts are split into the next row",
      "                                 ",
      "                                 ",
      "                                 ",
    ];

    run_lines(Arguments {
      terminal_size: size!(33, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 17)],
    });
  }

  #[test]
  fn new4() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "RSVIM!                         ",
      " a quite simple and small test ",
      "ll it contains several things w",
      "en the line is small enough to ",
      "en the line is too long to be c",
      "The extra parts are been trunca",
      "The extra parts are split into ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
    ];

    run_lines(Arguments {
      terminal_size: size!(31, 20),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 7)],
    });
  }

  #[test]
  fn update1() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect1 = vec![
      "Hello, RSVIM!        ",
      "This is a quite simpl",
      "But still it contains",
      "  1. When the line is",
      "  2. When the line is",
      "     * The extra part",
      "     * The extra part",
      "                     ",
      "                     ",
      "                     ",
    ];
    let expect2 = vec![
      "When the line is too ",
      "* The extra parts are",
      "* The extra parts are",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
    ];

    run_lines(Arguments {
      terminal_size: size!(21, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(false).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect1, expect2],
      viewport_start: vec![(0, 0), (4, 5)],
    });
  }
}

#[cfg(test)]
mod tests_wrap_nolinebreak {
  use super::*;

  #[test]
  fn new1() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "Hello, RSV",
      "IM!       ",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.      ",
      "But still ",
      "it contain",
      "s several ",
    ];

    run_lines(Arguments {
      terminal_size: size!(10, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new2() {
    let buffer_lines = vec![
      "\t\t\t* The extra parts are split\tinto the next row,\tif either line-wrap\tor word-wrap options are been set. If the extra parts are still too long to\t来放在下一个横行内，一遍又一遍的重复这样的操作。This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "                        * T",
      "he extra parts are split   ",
      "        into the next row, ",
      "        if either line-wrap",
      "        or word-wrap option",
      "s are been set. If the extr",
      "a parts are still too long ",
      "to        来放在下一个横行 ",
      "内，一遍又一遍的重复这样的 ",
      "操作。This operation also e",
    ];

    run_lines(Arguments {
      terminal_size: size!(27, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new3() {
    let expect = vec![
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
    ];

    run_empty(Arguments {
      terminal_size: size!(20, 9),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines: vec![],
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new4() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things\t我们想要测试的：\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
      "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "Hello, RSVIM!      ",
      "This is a quite sim",
      "ple and small test ",
      "lines.             ",
      "But still it contai",
      "ns several things  ",
      "        我们想要测 ",
      "试的：             ",
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
      "p doesn't affect th",
      "e rendering.       ",
      "        2. When the",
      " line is too long t",
      "o be completely put",
      " in a row of the wi",
      "ndow content widget",
      ", there're multiple",
      " cases:            ",
      "                *  ",
      "如果行换行和单词换 ",
      "行这两个选项都没有 ",
      "选中，那么这些超出 ",
      "窗口的文本内容会被 ",
      "截断。             ",
    ];

    run_lines(Arguments {
      terminal_size: size!(19, 30),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new5() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things\t我们想要测试的：\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
      "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "Hello, RSVIM!      ",
      "This is a quite sim",
      "ple and small test ",
      "lines.             ",
      "But still it contai",
      "ns several things  ",
      "        我们想要测 ",
      "试的：             ",
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
      "p doesn't affect th",
      "e rendering.       ",
      "        2. When the",
      " line is too long t",
      "o be completely put",
      " in a row of the wi",
      "ndow content widget",
      ", there're multiple",
      " cases:            ",
      "                *  ",
      "如果行换行和单词换 ",
      "行这两个选项都没有<",
    ];

    run_lines(Arguments {
      terminal_size: size!(19, 27),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn update1() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things\t我们想要测试的：\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
      "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect1 = vec![
      "Hello, RSVIM!      ",
      "This is a quite sim",
      "ple and small test ",
      "lines.             ",
      "But still it contai",
      "ns several things  ",
      "        我们想要测 ",
      "试的：             ",
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
    ];
    let expect2 = vec![
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
      "p doesn't affect th",
      "e rendering.       ",
      "        2. When the",
      " line is too long t",
      "o be completely put",
      " in a row of the wi",
      "ndow content widget",
      ", there're multiple",
    ];

    run_lines(Arguments {
      terminal_size: size!(19, 15),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default()
        .wrap(true)
        .line_break(false)
        .build()
        .unwrap(),
      buffer_lines,
      expect_canvas: vec![expect1, expect2],
      viewport_start: vec![(0, 0), (3, 0)],
    });
  }

  #[test]
  fn update2() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things\t我们想要测试的：\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
      "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "                * T",
      "he extra parts are ",
      "split into the next",
      " row, if either lin",
      "e-wrap or word-wrap",
      " options are been s",
      "et. If the extra pa",
      "rts are still too l",
      "ong to put in the n",
      "ext row, repeat thi",
      "s operation again a",
      "nd again. This oper",
      "ation also eats mor",
      "e rows in the windo",
      "w, thus it may cont",
    ];

    run_lines(Arguments {
      terminal_size: size!(19, 15),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default()
        .wrap(true)
        .line_break(false)
        .build()
        .unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(6, 0)],
    });
  }

  #[test]
  fn update3() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things\t我们想要测试的：\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
      "\t\t* The extra parts are split into the next row\n",
    ];
    let expect = vec![
      "                * T",
      "he extra parts are ",
      "split into the next",
      " row               ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
    ];

    run_lines(Arguments {
      terminal_size: size!(19, 15),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default()
        .wrap(true)
        .line_break(false)
        .build()
        .unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(6, 0)],
    });
  }
}

#[cfg(test)]
mod tests_wrap_nolinebreak_eol {
  use super::*;

  #[test]
  fn new1_crlf_win() {
    let buffer_lines = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple and small test lines123456.\r\n",
      "But still it contains several things we want to test:\r\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r\n",
    ];
    let expect = vec![
      "Hello, RSV",
      "IM!       ",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes123456.",
      "But still ",
      "it contain",
      "s several ",
    ];

    run_lines(Arguments {
      terminal_size: size!(10, 10),
      buffer_opts: BufferOptionsBuilder::default()
        .file_format(FileFormatOption::Dos)
        .build()
        .unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }

  #[test]
  fn new1_cr_mac() {
    let buffer_lines = vec![
      "Hello, RSVIM!\r",
      "This is a quite simple and small test lines123456.\r",
      "But still it contains several things we want to test:\r",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r",
    ];
    let expect = vec![
      "Hello, RSV",
      "IM!       ",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes123456.",
      "But still ",
      "it contain",
      "s several ",
    ];

    run_lines(Arguments {
      terminal_size: size!(10, 10),
      buffer_opts: BufferOptionsBuilder::default()
        .file_format(FileFormatOption::Mac)
        .build()
        .unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 0)],
    });
  }
}

#[cfg(test)]
mod tests_wrap_nolinebreak_startcol {
  use super::*;

  #[test]
  fn new1() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "l test lin",
      "es.       ",
      "hings we w",
      "ant to tes",
      "t:        ",
      "ugh to com",
      "pletely pu",
      "t inside a",
      " row of th",
      "e window c",
    ];

    run_lines(Arguments {
      terminal_size: size!(10, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(1, 31)],
    });
  }

  #[test]
  fn new2() {
    let buffer_lines = vec![
      "\t\t\t* The extra parts are split\tinto the next row,\tif either line-wrap\tor word-wrap options are been set. If the extra parts are still too long to\t来放在下一个横行内，一遍又一遍的重复这样的操作。This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      ">>>>>        * The extra pa",
      "rts are split        into t",
      "he next row,        if eith",
      "er line-wrap        or word",
      "-wrap options are been set.",
      " If the extra parts are sti",
      "ll too long to        来放 ",
      "在下一个横行内，一遍又一遍 ",
      "的重复这样的操作。This oper",
      "ation also eats more rows i",
    ];

    run_lines(Arguments {
      terminal_size: size!(27, 10),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 11)],
    });
  }

  #[test]
  fn new3() {
    let expect = vec![
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
    ];

    run_empty(Arguments {
      terminal_size: size!(20, 9),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines: vec![],
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 3)],
    });
  }

  #[test]
  fn new4() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things\t我们想要测试的：\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
      "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect = vec![
      "RSVIM!             ",
      " a quite simple and",
      " small test lines. ",
      "ll it contains seve",
      "ral things         ",
      "我们想要测试的：   ",
      ">1. When the line i",
      "s small enough to c",
      "ompletely put insid",
      "e a row of the wind",
      "ow content widget, ",
      "then the line-wrap ",
      "and word-wrap doesn",
      "'t affect the rende",
      "ring.              ",
      ">2. When the line i",
      "s too long to be co",
      "mpletely put in a r",
      "ow of the window co",
      "ntent widget, there",
      "'re multiple cases:",
      ">        * 如果行换",
      "行和单词换行这两个 ",
      "选项都没有选中，那 ",
      "么这些超出窗口的文 ",
      "本内容会被截断。   ",
      ">        * The extr",
      "a parts are split i",
      "nto the next row, i",
      "f either line-wrap ",
    ];

    run_lines(Arguments {
      terminal_size: size!(19, 30),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default().wrap(true).build().unwrap(),
      buffer_lines,
      expect_canvas: vec![expect],
      viewport_start: vec![(0, 7)],
    });
  }

  #[test]
  fn update1() {
    let buffer_lines = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things\t我们想要测试的：\n",
      "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
      "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ];
    let expect1 = vec![
      "Hello, RSVIM!      ",
      "This is a quite sim",
      "ple and small test ",
      "lines.             ",
      "But still it contai",
      "ns several things  ",
      "        我们想要测 ",
      "试的：             ",
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
    ];
    let expect2 = vec![
      ">>>>>>>1. When the ",
      "line is small enoug",
      "h to completely put",
      " inside a row of th",
      "e window content wi",
      "dget, then the line",
      "-wrap and word-wrap",
      " doesn't affect the",
      " rendering.        ",
      ">>>>>>>2. When the ",
      "line is too long to",
      " be completely put ",
      "in a row of the win",
      "dow content widget,",
      " there're multiple ",
    ];

    run_lines(Arguments {
      terminal_size: size!(19, 15),
      buffer_opts: BufferOptionsBuilder::default().build().unwrap(),
      window_opts: WindowOptionsBuilder::default()
        .wrap(true)
        .line_break(false)
        .build()
        .unwrap(),
      buffer_lines,
      expect_canvas: vec![expect1, expect2],
      viewport_start: vec![(0, 0), (3, 1)],
    });
  }

  #[test]
  fn update2() {
    test_log_init();

    let terminal_size = size!(19, 15);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things\t我们想要测试的：\n",
        "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
        "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "he extra parts are ",
      "split into the next",
      " row, if either lin",
      "e-wrap or word-wrap",
      " options are been s",
      "et. If the extra pa",
      "rts are still too l",
      "ong to put in the n",
      "ext row, repeat thi",
      "s operation again a",
      "nd again. This oper",
      "ation also eats mor",
      "e rows in the windo",
      "w, thus it may cont",
      "ains less lines in ",
    ];
    let viewport =
      make_viewport(terminal_size, win_opts, buffer.clone(), 6, 19);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn update3() {
    test_log_init();

    let terminal_size = size!(19, 15);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things\t我们想要测试的：\n",
        "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
        "\t\t* The extra parts are split into the next row\n",
      ],
    );

    let expect = vec![
      ">>>>        * The e",
      "xtra parts are spli",
      "t into the next row",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
      "                   ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 6, 4);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }
}

#[cfg(test)]
mod tests_wrap_linebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = size!(10, 10);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello,    ",
      "RSVIM!    ",
      "This is a ",
      "quite     ",
      "simple and",
      " small    ",
      "test lines",
      ".         ",
      "But still ",
      "it        ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 0);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = size!(27, 15);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!              ",
      "This is a quite simple and ",
      "small test lines.          ",
      "But still it contains      ",
      "several things we want to  ",
      "test:                      ",
      "  1. When the line is small",
      " enough to completely put  ",
      "inside a row of the window ",
      "content widget, then the   ",
      "line-wrap and word-wrap    ",
      "doesn't affect the         ",
      "rendering.                 ",
      "  2. When the line is too  ",
      "long to be completely put  ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 0);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = size!(20, 8);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_empty_buffer(terminal_size, buf_opts);
    let expect = vec![
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 0);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn new4() {
    test_log_init();

    let terminal_size = size!(13, 31);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, 那么行换行和单词换行选项都不会影响最终的渲染效果。\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!",
      "This is a    ",
      "quite simple ",
      "andsmalltestl",
      "ineswithoutev",
      "enanewlinebre",
      "akbecausewewa",
      "nttotesthowit",
      "willhappensif",
      "thereisaveryl",
      "ongwordthatca",
      "nnotbeenpplac",
      "einsidearowof",
      "thewindowcont",
      "ent.         ",
      "But still it ",
      "contains     ",
      "several      ",
      "things we    ",
      "want to test:",
      "  1. When the",
      " line is     ",
      "small enough ",
      "to completely",
      " put inside a",
      " row of the  ",
      "window       ",
      "content      ",
      "widget, 那么 ",
      "行换行和单词 ",
      "换行选项都不 ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 0);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn new5() {
    test_log_init();

    let terminal_size = size!(10, 10);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contai\tseveral things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello,    ",
      "RSVIM!    ",
      "This is a ",
      "quite     ",
      "simple and",
      " small    ",
      "test lines",
      ".         ",
      "But still ",
      "it contai ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 0);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn update1() {
    test_log_init();

    let terminal_size = size!(10, 10);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contai\tseveral things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello,    ",
      "RSVIM!    ",
      "This is a ",
      "quite     ",
      "simple and",
      " small    ",
      "test lines",
      ".         ",
      "But still ",
      "it contai ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 0);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);

    let expect = vec![
      "But still ",
      "it contai ",
      "          ",
      "several   ",
      "things we ",
      "want to   ",
      "test:     ",
      "  1. When ",
      "the line  ",
      "is small  ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 2, 0);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }
}

#[cfg(test)]
mod tests_wrap_linebreak_startcol {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = size!(10, 10);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "lo, RSVIM!",
      "s is a    ",
      "quite     ",
      "simple and",
      " small    ",
      "test lines",
      ".         ",
      " still it ",
      "contains  ",
      "several   ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 3);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = size!(27, 15);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "imple and small test lines.",
      "ains several things we want",
      " to test:                  ",
      "e is small enough to       ",
      "completely put inside a row",
      " of the window content     ",
      "widget, then the line-wrap ",
      "and word-wrap doesn't      ",
      "affect the rendering.      ",
      "e is too long to be        ",
      "completely put in a row of ",
      "the window content widget, ",
      "there're multiple cases:   ",
      "parts are been truncated if",
      " both line-wrap and word-  ",
    ];
    let viewport =
      make_viewport(terminal_size, win_opts, buffer.clone(), 1, 17);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = size!(20, 8);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_empty_buffer(terminal_size, buf_opts);
    let expect = vec![
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 5);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }

  #[test]
  fn new4() {
    test_log_init();

    let terminal_size = size!(13, 31);
    let buf_opts = BufferOptionsBuilder::default().build().unwrap();
    let win_opts = WindowOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, 那么行换行和单词换行选项都不会影响最终的渲染效果。\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      " RSVIM!      ",
      "s a quite    ",
      "simple       ",
      "andsmalltestl",
      "ineswithoutev",
      "enanewlinebre",
      "akbecausewewa",
      "nttotesthowit",
      "willhappensif",
      "thereisaveryl",
      "ongwordthatca",
      "nnotbeenpplac",
      "einsidearowof",
      "thewindowcont",
      "ent.         ",
      "ill it       ",
      "contains     ",
      "several      ",
      "things we    ",
      "want to test:",
      "hen the line ",
      "is small     ",
      "enough to    ",
      "completely   ",
      "put inside a ",
      "row of the   ",
      "window       ",
      "content      ",
      "widget, 那么 ",
      "行换行和单词 ",
      "换行选项都不 ",
    ];
    let viewport = make_viewport(terminal_size, win_opts, buffer.clone(), 0, 6);
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_canvas(&actual, &expect);
  }
}
