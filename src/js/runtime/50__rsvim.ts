/**
 * The global namespace for `Rsvim` specific, non-standard runtime APIs.
 *
 * @see [Vim: help.txt](https://vimhelp.org/)
 * @see [Neovim docs - Api](https://neovim.io/doc/user/api.html)
 * @see [Deno APIs](https://docs.deno.com/api/deno/)
 *
 * @packageDocumentation
 *
 * @categoryDescription Global Object
 * The global namespace.
 *
 * @categoryDescription Editor APIs
 * These APIs are specific for Rsvim editors such as buffers, windows, key mappings, etc.
 *
 * @categoryDescription General APIs
 * These APIs are general purpose for common JavaScript runtime, keeps the same with [Deno APIs](https://docs.deno.com/api/deno/).
 */

// @ts-ignore Ignore internal import warning
import infra from "rsvim:ext/infra";

/**
 * The `Rsvim` global object, it contains multiple sub fields:
 *
 * - `Rsvim.opt`: Global editor options.
 *
 * @category Global Object
 */
export interface Rsvim {
  opt: RsvimOpt;
}

/**
 * The `Rsvim.opt` namespace for global editor options.
 *
 * @category Editor APIs
 */
export interface RsvimOpt {
  /**
   * Get the _wrap_ option.
   *
   * Local to {@link Window}.
   *
   * This option changes how text is displayed.
   *
   * When `true` (on), lines longer than the width of the window will wrap and
   * displaying continues on the next line. When `false` (off) lines will not wrap
   * and only part of long lines will be displayed. When the cursor is
   * moved to a part that is not shown, the screen will scroll horizontally.
   *
   * The line will be broken in the middle of a word if necessary. See {@link getLineBreak | getLineBreak()}
   * to get the break at a word boundary.
   *
   * @see [Wikipedia - line wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap)
   * @see [Vim: options.txt - 'wrap'](https://vimhelp.org/options.txt.html#%27wrap%27)
   * @returns {boolean}
   * @defaultValue `true`.
   */
  getWrap(): boolean;

  /**
   * Set the _wrap_ option.
   *
   * @see {@link getWrap | getWrap()}
   *
   * @param {boolean} value - The _wrap_ option.
   * @throws {@link !Error} if value is not a boolean value.
   */
  setWrap(value: boolean): void;

  /**
   * Get the _line-break_ option.
   *
   * Local to {@link Window}.
   *
   * If `true` (on), Vim will wrap long lines at a character in 'breakat' rather
   * than at the last character that fits on the screen.
   *
   * It only affects the way the file is displayed, not its contents.
   * If 'breakindent' is set, line is visually indented. Then, the value
   * of 'showbreak' is used to put in front of wrapped lines. This option
   * is not used when the {@link getWrap() | _wrap_} option is `false`.
   *
   * @see [Wikipedia - word wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap)
   * @see [Vim: options.txt - 'linebreak'](https://vimhelp.org/options.txt.html#%27linebreak%27)
   *
   * @returns {boolean}
   * @defaultValue `false`.
   */
  getLineBreak(): boolean;

  /**
   * Set the _line-break_ option.
   *
   * @see {@link getLineBreak | getLineBreak()}
   *
   * @param {boolean} value - The _line-break_ option.
   * @throws {@link !Error} if value is not a boolean value.
   */
  setLineBreak(value: boolean): void;
}

(function (globalThis: { Rsvim: Rsvim }) {
  globalThis.Rsvim = {
    opt: {
      getWrap: function (): boolean {
        // @ts-ignore Ignore __InternalRsvimGlobalObject warning
        return __InternalRsvimGlobalObject.opt_get_wrap();
      },
      setWrap: function (value: boolean): void {
        if (typeof value !== "boolean") {
          throw new Error(
            `"Rsvim.opt.setWrap" value must be boolean type, but found ${infra.stringify(value)}`,
          );
        }
        // @ts-ignore Ignore __InternalRsvimGlobalObject warning
        __InternalRsvimGlobalObject.opt_set_wrap(value);
      },
      getLineBreak: function (): boolean {
        // @ts-ignore Ignore __InternalRsvimGlobalObject warning
        return __InternalRsvimGlobalObject.opt_get_line_break();
      },
      setLineBreak: function (value: boolean): void {
        if (typeof value !== "boolean") {
          throw new Error(
            `"Rsvim.opt.setLineBreak" value must be boolean type, but found ${infra.stringify(value)}`,
          );
        }
        // @ts-ignore Ignore __InternalRsvimGlobalObject warning
        __InternalRsvimGlobalObject.opt_set_line_break(value);
      },
    } as RsvimOpt,
  } as Rsvim;
})(globalThis as unknown as { Rsvim: Rsvim });
