/**
 * The global namespace for `Rsvim` specific, non-standard runtime APIs.
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
 *
 * @see [Vim: help.txt](https://vimhelp.org/)
 * @see [Neovim docs - Api](https://neovim.io/doc/user/api.html)
 * @see [Deno APIs](https://docs.deno.com/api/deno/)
 */

// @ts-ignore Ignore warning
import infra from "rsvim:ext/infra";

/**
 * The `Rsvim` global object, it contains multiple sub fields:
 *
 * - `Rsvim.opt`: Global editor options.
 *
 *
 * @example
 * ```javascript
 * // Create a variable alias to 'Rsvim'.
 * const vim = Rsvim;
 * ```
 *
 * @category Global Object
 * @hideconstructor
 */
export class Rsvim {
  readonly opt: RsvimOpt = new RsvimOpt();
}

/**
 * The `Rsvim.opt` object for global editor options.
 *
 * @example
 * ```javascript
 * // Create a variable alias to 'Rsvim.opt'.
 * const opt = Rsvim.opt;
 * ```
 *
 * @category Editor APIs
 * @hideconstructor
 */
export class RsvimOpt {
  /**
   * Get the _wrap_ option.
   *
   * Local to Window.
   *
   * This option changes how text is displayed.
   *
   * When `true` (on), lines longer than the width of the window will wrap and
   * displaying continues on the next line. When `false` (off) lines will not wrap
   * and only part of long lines will be displayed. When the cursor is
   * moved to a part that is not shown, the screen will scroll horizontally.
   *
   * The line will be broken in the middle of a word if necessary. See {@link lineBreak}
   * to get the break at a word boundary.
   *
   * @see [Wikipedia - line wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap)
   * @see [Vim: options.txt - 'wrap'](https://vimhelp.org/options.txt.html#%27wrap%27)
   *
   * @example
   * ```javascript
   * // Get the 'wrap' option.
   * const value = Rsvim.opt.wrap;
   * // Set the 'wrap' option.
   * Rsvim.opt.wrap = true;
   * ```
   *
   * @returns {boolean}
   * @defaultValue `true`
   */
  get wrap(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_wrap();
  }

  /**
   * Set the _wrap_ option.
   *
   * @param {boolean} value - The _wrap_ option.
   * @throws {@link !Error} if value is not a boolean value.
   */
  set wrap(value: boolean) {
    if (typeof value !== "boolean") {
      throw new Error(
        `"Rsvim.opt.wrap" value must be boolean type, but found ${infra.stringify(value)}`,
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_wrap(value);
  }

  /**
   * Get the _line-break_ option.
   *
   * Local to Window.
   *
   * If `true` (on), Vim will wrap long lines at a character in {@link breakAt} rather
   * than at the last character that fits on the screen.
   *
   * It only affects the way the file is displayed, not its contents.
   * If 'breakindent' is set, line is visually indented. Then, the value
   * of 'showbreak' is used to put in front of wrapped lines. This option
   * is not used when the {@link wrap} option is `false`.
   *
   * @see [Wikipedia - word wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap)
   * @see [Vim: options.txt - 'linebreak'](https://vimhelp.org/options.txt.html#%27linebreak%27)
   *
   * @example
   * ```javascript
   * // Get the 'lineBreak' option.
   * const value = Rsvim.opt.lineBreak;
   * // Set the 'lineBreak' option.
   * Rsvim.opt.lineBreak = true;
   * ```
   *
   * @returns {boolean}
   * @defaultValue `false`
   */
  get lineBreak(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_line_break();
  }

  /**
   * Set the _line-break_ option.
   *
   * @param {boolean} value - The _line-break_ option.
   * @throws {@link !Error} if value is not a boolean value.
   */
  set lineBreak(value: boolean) {
    if (typeof value !== "boolean") {
      throw new Error(
        `"Rsvim.opt.lineBreak" value must be boolean type, but found ${infra.stringify(value)}`,
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_line_break(value);
  }

  // /**
  //  * Get the _break-at_ option.
  //  *
  //  * Local to Window.
  //  *
  //  * This option lets you choose which characters might cause a line
  //  * break if {@link lineBreak} is `true` (on). Only works for ASCII and also for 8-bit
  //  * characters when {@link encoding} is an 8-bit encoding.
  //  *
  //  * @see {@link lineBreak}
  //  * @see [Vim: options.txt - 'breakat'](https://vimhelp.org/options.txt.html#%27breakat%27)
  //  *
  //  * @returns {string}
  //  * @defaultValue `" ^I!@*-+;:,./?"`
  //  */
  // get breakAt(): string {
  //   // @ts-ignore Ignore warning
  //   return __InternalRsvimGlobalObject.opt_get_break_at();
  // }
  //
  // /**
  //  * Set the _break-at_ option.
  //  *
  //  * @param {string} value - The _break-at_ option.
  //  * @throws {@link !Error} if value is not a string value.
  //  */
  // set breakAt(value: string) {
  //   if (typeof value !== "string") {
  //     throw new Error(
  //       `"Rsvim.opt.breakAt" value must be string type, but found ${infra.stringify(value)}`,
  //     );
  //   }
  //   // @ts-ignore Ignore warning
  //   __InternalRsvimGlobalObject.opt_set_break_at(value);
  // }
}

(function (globalThis: { Rsvim: Rsvim }) {
  globalThis.Rsvim = new Rsvim();
})(globalThis as unknown as { Rsvim: Rsvim });
