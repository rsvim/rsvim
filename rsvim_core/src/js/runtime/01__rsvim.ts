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
 * - `Rsvim.buf`: Buffer APIs.
 * - `Rsvim.cmd`: Ex command APIs.
 * - `Rsvim.opt`: Global options.
 * - `Rsvim.rt`: JavaScript runtime (editor process) APIs.
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
  readonly rt: RsvimRt = new RsvimRt();
}

/**
 * The `Rsvim.buf` global object for Vim buffers.
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
   * :::warning
   * When the editor is not initialized, i.e. there's no buffer/window created. It
   * will return `null`. Once the editor is initialized, there will always have a
   * valid buffer binded to the "current" window (where your cursor is). It will return
   * the valid buffer ID.
   * :::
   *
   * @returns {number | null} It returns a valid buffer ID if the editor is initialized.
   * Otherwise it returns `null` if the editor is not initialized.
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
   * :::warning
   * When the editor is not initialized, i.e. there's no buffer/window created. It
   * will return an empty array. Once the editor is initialized, there will have at least 1
   * buffer binded to the "current" window (where your cursor is). It will return all the
   * buffer IDs as an array.
   * :::
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
   *   const bytes = Rsvim.buf.writeSync(bufId);
   *   Rsvim.cmd.echo(`Buffer ${bufId} has been saved, ${bytes} bytes written`);
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
 * The `Rsvim.cmd` global object for Ex commands.
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
   * @param {message} message - It accepts string and other primitive types, except `null`
   * and `undefined`.
   *
   * @throws Throws {@link !Error} if no parameter provided, or the parameter is `null` or `undefined`.
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

type FileEncodingOption = "utf-8";
type FileFormatOption = "dos" | "unix" | "mac";

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
   * Get the _file-encoding_ option. Local to {@link Buffer}.
   *
   * Sets the [character encoding](https://en.wikipedia.org/wiki/Character_encoding) for the file of this buffer.
   * This will determine which character encoding is used when RSVIM read/write a file from file system.
   *
   * :::warning
   * For now, only **utf-8** encoding is supported.
   * :::
   *
   * @returns {FileEncodingOption}
   *
   * @defaultValue `"utf-8"`
   *
   * @example
   * ```javascript
   * // Get the 'file-encoding' option.
   * const value = Rsvim.opt.fileEncoding;
   * ```
   */
  get fileEncoding(): FileEncodingOption {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_file_encoding();
  }

  /**
   * Set the _file-encoding_ option.
   *
   * @param {FileEncodingOption} value - The _file-encoding_ option.
   * @throws Throws {@link !Error} if value is not a valid option.
   *
   * @example
   * ```javascript
   * // Set the 'file-encoding' option.
   * Rsvim.opt.fileEncoding = "utf-8";
   * ```
   */
  set fileEncoding(value: FileEncodingOption) {
    if (value !== "utf-8") {
      throw new Error(
        `"Rsvim.opt.fileEncoding" parameter must be a valid option, but found ${value} (${typeof value})`,
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_file_encoding(value);
  }

  /**
   * Get the _file-format_ option. Local to {@link Buffer}.
   *
   * Sets the [line end](https://en.wikipedia.org/wiki/Newline) for the file of this buffer. There are 3 kinds of line end:
   * - `CRLF`: used by [Windows](https://www.microsoft.com/windows).
   * - `LF`: used by [Linux](https://en.wikipedia.org/wiki/Linux) and [Unix](https://en.wikipedia.org/wiki/Unix) (include [MacOS](https://www.apple.com/macos/)).
   * - `CR`: used by [classic MacOS](https://en.wikipedia.org/wiki/Classic_Mac_OS). Today's Mac also uses `LF` as line end, you would never use `CR` any more.
   *
   * :::note
   * In fact it should be named to "line-end", it is called "file-format" just to be consistent
   * with Vim's [fileformat](https://vimhelp.org/options.txt.html#%27fileformat%27) option.
   * :::
   *
   * For this API, it has below options:
   * - `"dos"`: equivalent to `CRLF` line end.
   * - `"unix"`: equivalent to `LF` line end.
   * - `"mac"`: equivalent to `CR` line end. You would never use it today.
   *
   * @returns {FileFormatOption}
   *
   * @defaultValue `"dos"` for Windows/MS-DOS, `"unix"` for Linux/Unix/MacOS.
   *
   * @example
   * ```javascript
   * // Get the 'file-format' option.
   * const value = Rsvim.opt.fileFormat;
   * ```
   */
  get fileFormat(): FileFormatOption {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_file_format();
  }

  /**
   * Set the _file-format_ option.
   *
   * @param {FileFormatOption} value - The _file-format_ option.
   * @throws Throws {@link !Error} if value is not a valid option.
   *
   * @example
   * ```javascript
   * // Set the 'file-format' option.
   * Rsvim.opt.fileFormat = "unix";
   * ```
   */
  set fileFormat(value: FileFormatOption) {
    if (value !== "dos" && value !== "unix" && value !== "mac") {
      throw new Error(
        `"Rsvim.opt.fileFormat" parameter must be a valid option, but found ${value} (${typeof value})`,
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_file_format(value);
  }

  /**
   * Get the _line-break_ option. This options is also known as
   * [word wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap). Local to {@link Window}.
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
   * Get the _tab-stop_ option. This option is also known as
   * [tab-size](https://developer.mozilla.org/en-US/docs/Web/CSS/tab-size).
   * Local to {@link Buffer}.
   *
   * This option changes how text is displayed.
   *
   * Defines how many columns (on the terminal) used to display the
   * [horizontal tab](https://en.wikipedia.org/wiki/Tab_key) (ASCII `9`). This value should be between `[1,65535]`.
   *
   *
   * @returns {number}
   *
   * @defaultValue `8`
   *
   * @example
   * ```javascript
   * // Get the 'tab-stop' option.
   * const value = Rsvim.opt.tabStop;
   * ```
   */
  get tabStop(): number {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_tab_stop();
  }

  /**
   * Set the _tab-stop_ option.
   *
   * @param {number} value - The _tab-stop_ option. It only accepts an integer between `[1,65535]`.
   * @throws Throws {@link !Error} if value is not a integer value, or the integer value is not between `[1,65535]`.
   *
   * @example
   * ```javascript
   * // Set the 'tab-stop' option.
   * Rsvim.opt.tabStop = 4;
   * ```
   */
  set tabStop(value: number) {
    if (typeof value !== "number" || value < 1 || value > 65535) {
      throw new Error(
        `"Rsvim.opt.tabStop" parameter must be an integer value between [1,65535], but found ${value} (${typeof value})`,
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_tab_stop(value);
  }

  /**
   * Get the _wrap_ option. This option is also known as
   * [line wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap). Local to {@link Window}.
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

/**
 * The `Rsvim.rt` global object for javascript runtime (editor process).
 *
 * @example
 * ```javascript
 * // Create a alias to 'Rsvim.rt'.
 * const rt = Rsvim.rt;
 * ```
 *
 * @category General APIs
 * @hideconstructor
 */
export class RsvimRt {
  /**
   * Exit editor.
   *
   * :::tip
   * To ensure file system data safety, editor will wait for all the ongoing file write operations
   * to complete before actually exiting, however any new write requests will be rejected.
   * :::
   *
   * @param {exitCode?} exitCode - The editor process exit with this exit code. This parameter can be omitted,
   * by default it uses `0` to indicate no error happens.
   *
   * @throws Throws {@link !Error} if `exitCode` parameter is neither a integer nor `undefined`.
   *
   * @example
   * ```javascript
   * // Exit with default exit code `0`.
   * Rsvim.rt.exit();
   *
   * // Exit with error exit code `-1`.
   * Rsvim.rt.exit(-1);
   * ```
   */
  public exit(exitCode?: number) {
    if (exitCode !== undefined && typeof exitCode !== "number") {
      throw new Error(
        '"Rsvim.rt.exit" exit code parameter must be a valid integer or undefined',
      );
    }
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.rt_exit(exitCode);
  }
}

(function (globalThis: { Rsvim: Rsvim }) {
  globalThis.Rsvim = new Rsvim();
})(globalThis as unknown as { Rsvim: Rsvim });
