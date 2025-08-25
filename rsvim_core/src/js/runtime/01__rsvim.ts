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
 * - `Rsvim.buf`: Buffers.
 * - `Rsvim.cmd`: Commands.
 * - `Rsvim.opt`: Options.
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
  readonly buf: RsvimBuf = new RsvimBuf();
  readonly cmd: RsvimCmd = new RsvimCmd();
  readonly opt: RsvimOpt = new RsvimOpt();
}

/**
 * The `Rsvim.buf` global object for buffers.
 *
 * @example
 * ```javascript
 * // Create a alias to 'Rsvim.buf'.
 * const buf = Rsvim.buf;
 * ```
 *
 * @category Editor APIs
 * @hideconstructor
 */
export class RsvimBuf {
  /**
   * Get current buffer's ID.
   *
   * The "current" buffer is the buffer that the window where your cursor is
   * located is binded to. See {@link RsvimWin}.
   *
   * @returns {number | null} It returns `null` before the editor is
   * initialized since there's no buffer/window created. Once the editor is
   * initialized, it always returns a valid buffer ID `number`, since there
   * will always have a valid buffer binded to the current window (where your
   * cursor is).
   *
   * @example
   * ```javascript
   * const bufId = Rsvim.buf.current();
   * ```
   */
  public current(): number | null {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.buf_current();
  }

  /**
   * List all buffers' IDs.
   *
   * @returns {number[]} All the buffers' IDs as an array. If there's no
   * buffer (i.e. the editor is not initialized), it returns an empty array.
   *
   * @example
   * ```javascript
   * const bufIds = Rsvim.buf.list();
   * ```
   */
  public list(): number[] {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.buf_list();
  }

  /**
   * Write (save) buffer's text contents to local filesystem synchronizely.
   *
   * @param {number} bufId - The buffer's ID that you want to write to filesystem.
   *
   * @returns {number} It returns a positive integer to indicate how many bytes
   * have been written to the file, if written successfully.
   *
   * @throws Throws {@link !Error} if failed to write buffer contents to file system.
   *
   * @example
   * ```javascript
   * const bufId = Rsvim.buf.currentBufferId();
   * try {
   *   Rsvim.buf.writeSync(bufId);
   * } catch (e) {
   *   Rsvim.cmd.echo(`Error: failed to save buffer ${bufId}, exception: ${e}`);
   * }
   * ```
   */
  public writeSync(bufId: number): number {
    if (typeof bufId !== "number") {
      throw new Error(
        `"Rsvim.buf.write" bufId parameter must be a integer value, but found ${bufId} (${typeof bufId})`,
      );
    }
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.buf_write_sync(bufId);
  }
}

/**
 * The `Rsvim.cmd` global object for ex commands.
 *
 * :::tip
 * The "ex command" mostly describes the product function, i.e. when user types ":" in normal mode,
 * user can move cursor to command-line and input commands. Rather than referring to the
 * ["ex commands"](https://vimhelp.org/intro.txt.html#Ex-mode) in Vim editor.
 * :::
 *
 * @example
 * ```javascript
 * // Create a alias to 'Rsvim.cmd'.
 * const cmd = Rsvim.cmd;
 * ```
 *
 * @category Editor APIs
 * @hideconstructor
 */
export class RsvimCmd {
  /**
   * Echo message to the command-line.
   *
   * @example
   * ```javascript
   * Rsvim.cmd.echo("Hello Rsvim!");
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
   * Get the _line-break_ option. This options is also known as [word wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap).
   *
   * Local to Window.
   *
   * If `true`, Vim will wrap long lines by a word boundary rather than at the last character that fits on the screen.
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

  /**
   * Get the _wrap_ option. This option is also known as [line wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap).
   *
   * Local to Window.
   *
   * This option changes how text is displayed.
   *
   * When `true`, lines longer than the width of the window will wrap and
   * displaying continues on the next line. When `false` lines will not wrap
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
}

(function (globalThis: { Rsvim: Rsvim }) {
  globalThis.Rsvim = new Rsvim();
})(globalThis as unknown as { Rsvim: Rsvim });
