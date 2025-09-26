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
 */
export interface Rsvim {
  readonly buf: RsvimBuf;
  readonly cmd: RsvimCmd;
  readonly opt: RsvimOpt;
  readonly rt: RsvimRt;
}

class RsvimImpl implements Rsvim {
  buf = new RsvimBufImpl();
  cmd = new RsvimCmdImpl();
  opt = new RsvimOptImpl();
  rt = new RsvimRtImpl();
}

function checkNotNull(arg: any, msg: string) {
  if (arg === undefined || arg === null) {
    throw new TypeError(`${msg} cannot be undefined or null`);
  }
}

function checkIsNumber(arg: any, msg: string) {
  if (typeof arg !== "number") {
    throw new TypeError(`${msg} must be a number, but found ${typeof arg}`);
  }
}

function checkIsInteger(arg: any, msg: string) {
  checkIsNumber(arg, msg);
  if (!Number.isInteger(arg)) {
    throw new TypeError(`${msg} must be an integer, but found ${typeof arg}`);
  }
}

function checkIsBoolean(arg: any, msg: string) {
  if (typeof arg !== "boolean") {
    throw new TypeError(`${msg} must be a boolean, but found ${typeof arg}`);
  }
}

function checkIsString(arg: any, msg: string) {
  if (typeof arg !== "string") {
    throw new TypeError(`${msg} must be a string, but found ${typeof arg}`);
  }
}

function checkMatchPattern(arg: any, pat: RegExp, msg: string) {
  checkIsString(arg, msg);
  if (!pat.test(arg)) {
    throw new Error(`${msg} is invalid pattern: ${arg}"`);
  }
}

function checkIsFunction(arg: any, msg: string) {
  if (typeof arg !== "function") {
    throw new TypeError(`${msg} must be a function, but found ${typeof arg}`);
  }
}

function checkIsOptions(arg: any, options: any[], msg: string) {
  if (!options.includes(arg)) {
    throw new RangeError(`${msg} is invalid option: ${arg}`);
  }
}

function checkObjectContains(
  arg: any,
  fieldCheckers: { [index: string]: (arg: any, msg: string) => void },
  msg: string,
) {
  if (typeof arg !== "object") {
    throw new TypeError(`${msg} must be an object, but found ${typeof arg}`);
  }
  Object.entries(fieldCheckers).forEach(([field, checker]) => {
    if (Object.hasOwn(arg, field)) {
      checker(arg[field], msg);
    }
  });
}

function boundByIntegers(arg: any, bound: [number, number]) {
  if (arg < bound[0]) {
    return bound[0];
  }
  if (arg > bound[1]) {
    return bound[1];
  }
  return arg;
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
 */
export interface RsvimBuf {
  /**
   * Get current buffer's ID.
   *
   * The "current" buffer is the buffer that the window where your cursor is
   * located is binded to.
   *
   * :::warning
   * When the editor is not initialized, i.e. there's no buffer/window created. It
   * will return `undefined`. Once the editor is initialized, there will always have a
   * valid buffer binded to the "current" window (where your cursor is). It will return
   * the valid buffer ID.
   * :::
   *
   * @returns {number | undefined} It returns a valid buffer ID if the editor is initialized.
   * Otherwise it returns `undefined` if the editor is not initialized.
   *
   * @example
   * ```javascript
   * const bufId = Rsvim.buf.current();
   * ```
   */
  current(): number | undefined;

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
  list(): number[];

  /**
   * Write (save) buffer's text contents to local filesystem synchronizely.
   *
   * @param {number} bufId - The buffer's ID that you want to write to filesystem.
   *
   * @returns {number} It returns a positive integer to indicate how many bytes
   * have been written to the file, if written successfully.
   *
   * @throws Throws {@link !TypeError} if the parameter is invalid, or {@link !Error} if failed to write buffer to file system.
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
  writeSync(bufId: number): number;
}

class RsvimBufImpl implements RsvimBuf {
  current(): number | undefined {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.buf_current();
  }

  list(): number[] {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.buf_list();
  }

  writeSync(bufId: number): number {
    checkIsInteger(bufId, `"Rsvim.buf.writeSync" bufId`);

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
 */
export interface RsvimCmd {
  /**
   * Create a ex command with a callback function.
   *
   * :::warning
   * The only builtin command `js` cannot be override.
   * :::
   *
   * @param {string} name - Command name that is going to create. Only letters (`a-z` and `A-Z`), digits (`0-9`), underscore (`_`) and exclamation (`!`) are allowed in a command name. Command name must not begin with a digit.
   * @param {RsvimCmd.CommandCallback} callback - The backend logic that implements the command. It accepts an `ctx` parameter that contains all the information when user is running it. See {@link RsvimCmd.CommandCallback}.
   * @param {RsvimCmd.CommandAttributes} attributes - Attributes that control the command behavior. This parameter can be omitted, it will use the default attributes, see {@link RsvimCmd.CommandAttributes}.
   * @param {RsvimCmd.CommandOptions} options - Options that control how the command is created. This parameter can be omitted, it will use the default options, see {@link RsvimCmd.CommandOptions}.
   * @returns {undefined | RsvimCmd.CommandDefinition} It returns `undefined` is the command is newly created, or a command definition that was defined previously.
   *
   * @throws Throws {@link !TypeError} if any parameters are invalid.
   *
   * @example
   * ```javascript
   * function write(ctx: any): void {
   *   try {
   *     const bytes = Rsvim.buf.writeSync(bufId);
   *     Rsvim.cmd.echo(`Buffer ${bufId} has been saved, ${bytes} bytes written`);
   *   } catch (e) {
   *     Rsvim.cmd.echo(`Error: failed to save buffer ${bufId}, exception: ${e}`);
   *   }
   * }
   * Rsvim.cmd.create("write", write);
   * ```
   */
  create(
    name: string,
    callback: RsvimCmd.CommandCallback,
    attributes?: RsvimCmd.CommandAttributes,
    options?: RsvimCmd.CommandOptions,
  ): RsvimCmd.CommandDefinition | undefined;

  /**
   * Echo message to the command-line.
   *
   * @param {any} message - It accepts string and other primitive types, except `null` and `undefined`.
   *
   * @throws Throws {@link !TypeError} if the parameter is `null` or `undefined` or no parameter provided.
   *
   * @example
   * ```javascript
   * Rsvim.cmd.echo("Hello Rsvim!");
   * ```
   */
  echo(message: any): void;

  /**
   * List all registered ex commands. Note: The builtin `js` command will not be listed here.
   *
   * @returns {RsvimCmd.CommandDefinition[]} Returns all registered ex commands, except the `js` command.
   *
   * @example
   * ```javascript
   * Rsvim.cmd.list().forEach((cmd) => {
   *   Rsvim.cmd.echo(`Command: ${cmd.name}`);
   * });
   * ```
   */
  list(): RsvimCmd.CommandDefinition[];

  /**
   * Remove an ex command by name.
   *
   * :::warning
   * The only builtin command `js` cannot be removed.
   * :::
   *
   * @param {string} name - The command name to be removed.
   * @returns {RsvimCmd.CommandDefinition | undefined} Returns the removed {@link RsvimCmd.CommandDefinition}, or `undefined` if no command is been removed.
   *
   * @example
   * ```javascript
   * Rsvim.cmd.list().forEach((cmd) => {
   *   // Remove all registered commands.
   *   Rsvim.cmd.remove(cmd.name);
   * });
   * ```
   */
  remove(name: string): RsvimCmd.CommandDefinition | undefined;
}

class RsvimCmdImpl implements RsvimCmd {
  create(
    name: string,
    callback: RsvimCmd.CommandCallback,
    attributes?: RsvimCmd.CommandAttributes,
    options?: RsvimCmd.CommandOptions,
  ): RsvimCmd.CommandDefinition | undefined {
    checkMatchPattern(
      name,
      /^[A-Za-z_!][A-Za-z0-9_!]+$/,
      `"Rsvim.cmd.create" name`,
    );

    checkIsFunction(callback, `"Rsvim.cmd.create" callback`);

    if (attributes === undefined) {
      attributes = {};
    }
    checkObjectContains(
      attributes,
      {
        bang: checkIsBoolean,
        nargs: (arg: any, msg: string): void =>
          checkIsOptions(arg, ["0", "1", "?", "+", "*"], msg),
      },
      `"Rsvim.cmd.create" attributes`,
    );

    if (options === undefined) {
      options = {};
    }
    checkObjectContains(
      options,
      {
        force: checkIsBoolean,
      },
      `"Rsvim.cmd.create" options`,
    );

    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.cmd_create(
      name,
      callback,
      attributes,
      options,
    );
  }

  echo(message: any): void {
    checkNotNull(message, `"Rsvim.cmd.echo" message`);

    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.cmd_echo(message);
  }

  list(): RsvimCmd.CommandDefinition[] {
    return [];
  }
}

export namespace RsvimCmd {
  /**
   * Command attributes.
   *
   * @see {@link RsvimCmd.create}
   */
  export type CommandAttributes = {
    /**
     * Whether the command can take a `!` modifier, for example: `:w!`, `:qall!`.
     *
     * By default is `false`
,    */
    bang?: boolean;

    /**
     * Whether The command can take any arguments, and how many it can take:
     *
     * - `0`: No arguments are allowed.
     * - `1`: Exactly 1 argument is required.
     * - `*`: Any number of arguments are allowed, i.e. 0, 1 or more.
     * - `?`: 0 or 1 arguments are allowed.
     * - `+`: At least 1 arguments are required.
     *
     * By default is `"0"`
,    */
    nargs?: "0" | "1" | "*" | "+" | "?";
  };

  /**
   * Command options when creating a command.
   *
   * @see {@link RsvimCmd.create}
   */
  export type CommandOptions = {
    /**
     * Whether force override the command if there's already an existing one.
     *
     * By default is `true`
     */
    force?: boolean;
  };

  /**
   * Command callback function, this is the backend logic that implements a user ex command.
   *
   * @see {@link RsvimCmd.create}
,  */
  export type CommandCallback = (ctx: any) => void;

  /**
   * Command definition.
   */
  export type CommandDefinition = {
    name: string;
    callback: CommandCallback;
    attributes: CommandAttributes;
    options: CommandOptions;
  };
}

/**
 * @inline
 */
type FileEncodingOption = "utf-8";

/**
 * @inline
 */
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
 */
export interface RsvimOpt {
  /**
   * Get the _expand-tab_ option. Local to buffer.
   *
   * When in insert mode, inserts [spaces](https://en.wikipedia.org/wiki/Whitespace_character) (ASCII `32`)
   * instead of a [horizontal tab](https://en.wikipedia.org/wiki/Tab_key) (ASCII `9`).
   *
   * See {@link shiftWidth} to get the number of spaces when inserting.
   *
   * @returns {boolean}
   *
   * @defaultValue `false`
   *
   * @example
   * ```javascript
   * // Get the 'expand-tab' option.
   * const value = Rsvim.opt.expandTab;
   * ```
   */
  get expandTab(): boolean;

  /**
   * Set the _expand-tab_ option.
   *
   * @param {boolean} value - The _expand-tab_ option.
   * @throws Throws {@link !TypeError} if value is not a boolean.
   *
   * @example
   * ```javascript
   * // Set the 'expand-tab' option.
   * Rsvim.opt.expandTab = true;
   * ```
   */
  set expandTab(value: boolean);

  /**
   * Get the _file-encoding_ option. Local to buffer.
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
  get fileEncoding(): FileEncodingOption;

  /**
   * Set the _file-encoding_ option.
   *
   * @param {FileEncodingOption} value - The _file-encoding_ option.
   * @throws Throws {@link !RangeError} if value is an invalid option.
   *
   * @example
   * ```javascript
   * // Set the 'file-encoding' option.
   * Rsvim.opt.fileEncoding = "utf-8";
   * ```
   */
  set fileEncoding(value: FileEncodingOption);

  /**
   * Get the _file-format_ option. Local to buffer.
   *
   * Sets the [line end](https://en.wikipedia.org/wiki/Newline) for the buffer. There are 3 kinds of line end:
   * - `CRLF`: used by [Windows](https://www.microsoft.com/windows).
   * - `LF`: used by [Linux](https://en.wikipedia.org/wiki/Linux) and [Unix](https://en.wikipedia.org/wiki/Unix) (include [MacOS](https://www.apple.com/macos/)).
   * - `CR`: used by [classic MacOS](https://en.wikipedia.org/wiki/Classic_Mac_OS).
   *
   * :::warning
   * Today's Mac also uses `LF` as line end, you should never use `CR` any more.
   * :::
   *
   * :::note
   * In fact this option should be named to "line-end", "file-format" is just to be consistent
   * with Vim's [fileformat](https://vimhelp.org/options.txt.html#%27fileformat%27).
   * :::
   *
   * For this option, it has below choices:
   * - `"dos"`: equivalent to `CRLF` line end.
   * - `"unix"`: equivalent to `LF` line end.
   * - `"mac"`: equivalent to `CR` line end.
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
  get fileFormat(): FileFormatOption;

  /**
   * Set the _file-format_ option.
   *
   * @param {FileFormatOption} value - The _file-format_ option.
   * @throws Throws {@link !RangeError} if value is an invalid option.
   *
   * @example
   * ```javascript
   * // Set the 'file-format' option.
   * Rsvim.opt.fileFormat = "unix";
   * ```
   */
  set fileFormat(value: FileFormatOption);

  /**
   * Get the _line-break_ option. This options is also known as
   * [word wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap). Local to window.
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
  get lineBreak(): boolean;

  /**
   * Set the _line-break_ option.
   *
   * @param {boolean} value - The _line-break_ option.
   * @throws Throws {@link !TypeError} if value is not a boolean.
   *
   * @example
   * ```javascript
   * // Set the 'lineBreak' option.
   * Rsvim.opt.lineBreak = true;
   * ```
   */
  set lineBreak(value: boolean);

  /**
   * Get the _shift-width_ option. Local to buffer.
   *
   * When {@link expandTab} is `true`, the number of spaces that is used when inserts a
   * [horizontal tab](https://en.wikipedia.org/wiki/Tab_key) (ASCII `9`).
   *
   * When {@link expandTab} is `false`, this option is not been used.
   *
   *
   * @returns {number}
   *
   * @defaultValue `8`
   *
   * @example
   * ```javascript
   * // Get the 'shift-width' option.
   * const value = Rsvim.opt.shiftWidth;
   * ```
   */
  get shiftWidth(): number;

  /**
   * Set the _expand-tab_ option. It only accepts an integer between `[1,255]`, if the value is out of range, it will be bound to this range.
   *
   *
   * @param {boolean} value - The _expand-tab_ option.
   * @throws Throws {@link !TypeError} if value is not an integer.
   *
   * @example
   * ```javascript
   * // Set the 'shift-width' option.
   * Rsvim.opt.shiftWidth = 4;
   * ```
   */
  set shiftWidth(value: number);

  /**
   * Get the _tab-stop_ option. This option is also known as
   * [tab-size](https://developer.mozilla.org/en-US/docs/Web/CSS/tab-size).
   * Local to buffer.
   *
   * This option changes how text is displayed.
   *
   * Defines how many columns (on the terminal) used to display the
   * [horizontal tab](https://en.wikipedia.org/wiki/Tab_key) (ASCII `9`). This value should be between `[1,255]`.
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
  get tabStop(): number;

  /**
   * Set the _tab-stop_ option. It only accepts an integer between `[1,255]`, if the value is out of range, it will be bound to this range.
   *
   * @param {number} value - The _tab-stop_ option.
   * @throws Throws {@link !TypeError} if value is not an integer.
   *
   * @example
   * ```javascript
   * // Set the 'tab-stop' option.
   * Rsvim.opt.tabStop = 4;
   * ```
   */
  set tabStop(value: number);

  /**
   * Get the _wrap_ option. This option is also known as
   * [line wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap). Local to window.
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
  get wrap(): boolean;

  /**
   * Set the _wrap_ option.
   *
   * @param {boolean} value - The _wrap_ option.
   * @throws Throws {@link !TypeError} if value is not a boolean.
   *
   * @example
   * ```javascript
   * // Set the 'wrap' option.
   * Rsvim.opt.wrap = true;
   * ```
   */
  set wrap(value: boolean);
}

class RsvimOptImpl implements RsvimOpt {
  get expandTab(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_expand_tab();
  }

  set expandTab(value: boolean) {
    checkIsBoolean(value, `"Rsvim.opt.expandTab" value`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_expand_tab(value);
  }

  get fileEncoding(): FileEncodingOption {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_file_encoding();
  }

  set fileEncoding(value: FileEncodingOption) {
    checkIsOptions(value, ["utf-8"], `"Rsvim.opt.fileEncoding" value`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_file_encoding(value);
  }

  get fileFormat(): FileFormatOption {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_file_format();
  }

  set fileFormat(value: FileFormatOption) {
    checkIsOptions(
      value,
      ["dos", "unix", "mac"],
      `"Rsvim.opt.fileFormat" value`,
    );
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_file_format(value);
  }

  get lineBreak(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_line_break();
  }

  set lineBreak(value: boolean) {
    checkIsBoolean(value, `"Rsvim.opt.lineBreak" value`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_line_break(value);
  }

  get shiftWidth(): number {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_shift_width();
  }

  set shiftWidth(value: number) {
    checkIsInteger(value, `"Rsvim.opt.shiftWidth" value`);
    value = boundByIntegers(value, [1, 255]);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_shift_width(value);
  }

  get tabStop(): number {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_tab_stop();
  }

  set tabStop(value: number) {
    checkIsInteger(value, `"Rsvim.opt.tabStop" value`);
    value = boundByIntegers(value, [1, 255]);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_tab_stop(value);
  }

  get wrap(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_wrap();
  }

  set wrap(value: boolean) {
    checkIsBoolean(value, `"Rsvim.opt.wrap" value`);
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
 */
export interface RsvimRt {
  /**
   * Exit editor.
   *
   * :::tip
   * To ensure file system data safety, editor will wait for all the ongoing file write operations
   * to complete before actually exiting, however any new write requests will be rejected.
   * :::
   *
   * @param {exitCode?} exitCode - The editor process exit with this exit code. This parameter can be omitted,
   * by default uses `0` to indicate no error.
   *
   * @throws Throws {@link !TypeError} if `exitCode` is neither an integer nor `undefined`.
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
  exit(exitCode?: number): void;
}

class RsvimRtImpl implements RsvimRt {
  exit(exitCode?: number): void {
    if (exitCode === undefined) {
      exitCode = 0;
    }
    checkIsInteger(exitCode, `"Rsvim.rt.exit" exit code`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.rt_exit(exitCode);
  }
}

(function (globalThis: { Rsvim: Rsvim }) {
  globalThis.Rsvim = new RsvimImpl();
})(globalThis as unknown as { Rsvim: Rsvim });
