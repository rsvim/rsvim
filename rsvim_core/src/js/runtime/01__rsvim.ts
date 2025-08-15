/**
 * The `Rsvim` global object, it contains two groups:
 *
 * - General APIs.
 * - Editor APIs.
 *
 * @packageDocumentation
 *
 * @categoryDescription Global Object
 * The global object.
 *
 * @categoryDescription Editor APIs
 * These APIs are specific for editor, such as buffers, windows, key mappings, etc.
 *
 * @categoryDescription General APIs
 * These APIs are general for common javascript-based runtime, similar to [Deno APIs](https://docs.deno.com/api/deno/).
 */

/**
 * The `Rsvim` global object, it contains multiple sub fields:
 *
 * - `Rsvim.opt`: Global editor options.
 *
 * @example
 * ```javascript
 * // Create a alias to 'Rsvim'.
 * const vim = Rsvim;
 * ```
 *
 * @category Global Object
 * @hideconstructor
 */
export class Rsvim {
  readonly cmd: RsvimCmd = new RsvimCmd();
  readonly opt: RsvimOpt = new RsvimOpt();
}

/**
 * The `Rsvim.cmd` global object for rsvim core commands.
 *
 * @example
 * ```javascript
 * const cmd = Rsvim.cmd;
 * ```
 *
 * @category Editor APIs
 * @hideconstructor
 */
export class RsvimCmd {
  /**
   * Echo message to the command line widget.
   *
   * @example
   * ```javascript
   * Rsvim.cmd.echo("A message to Rsvim !");
   * ```
   */
  public echo(message: string) {
    if (message === undefined || message === null) {
      throw new Error(
        '"Rsvim.cmd.echo" message parameter cannot be undefined or null',
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.cmd_echo(message);
  }
}

/**
 * The `Rsvim.opt` global object for global editor options.
 *
 * @example
 * ```javascript
 * // Create a alias to 'Rsvim.opt'.
 * const opt = Rsvim.opt;
 * ```
 *
 * @category Editor APIs
 * @hideconstructor
 */
export class RsvimOpt {
  /**
   * Get the _wrap_ option. This option is also known as [line wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap).
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
   * @returns {boolean}
   *
   * @defaultValue `true`
   *
   * @example
   * ```javascript
   * // Get the 'wrap' option.
   * const value = Rsvim.opt.wrap;
   * ```
   */
  get wrap(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_wrap();
  }

  /**
   * Set the _wrap_ option.
   *
   * @param {boolean} value - The _wrap_ option.
   * @throws Throws {@link !Error} if value is not a boolean value.
   *
   * @example
   * ```javascript
   * // Set the 'wrap' option.
   * Rsvim.opt.wrap = true;
   * ```
   */
  set wrap(value: boolean) {
    if (typeof value !== "boolean") {
      throw new Error(
        `"Rsvim.opt.wrap" must be a boolean value, but found ${value} (${typeof value})`,
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_wrap(value);
  }

  /**
   * Get the _line-break_ option. This options is also known as [word wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap).
   *
   * Local to Window.
   *
   * If `true` (on), Vim will wrap long lines by a word boundary rather than at the last character that fits on the screen.
   * It only affects the way the file is displayed, not its contents.
   *
   * This option is not used when the {@link wrap} option is `false`.
   *
   * @returns {boolean}
   *
   * @defaultValue `false`
   *
   * @example
   * ```javascript
   * // Get the 'lineBreak' option.
   * const value = Rsvim.opt.lineBreak;
   * ```
   */
  get lineBreak(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_line_break();
  }

  /**
   * Set the _line-break_ option.
   *
   * @param {boolean} value - The _line-break_ option.
   * @throws Throws {@link !Error} if value is not a boolean value.
   *
   * @example
   * ```javascript
   * // Set the 'lineBreak' option.
   * Rsvim.opt.lineBreak = true;
   * ```
   */
  set lineBreak(value: boolean) {
    if (typeof value !== "boolean") {
      throw new Error(
        `"Rsvim.opt.lineBreak" must be a boolean value, but found ${value} (${typeof value})`,
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_line_break(value);
  }
}

(function (globalThis: { Rsvim: Rsvim }) {
  globalThis.Rsvim = new Rsvim();
})(globalThis as unknown as { Rsvim: Rsvim });
