/**
 * ---
 * title: Rsvim API
 * sidebar_position: 2
 * ---
 *
 * The `Rsvim` global object, it contains two groups:
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

/** @hidden */
function isNull(arg: any): boolean {
  return arg === undefined || arg === null;
}

/** @hidden */
function isString(arg: any): boolean {
  return typeof arg === "string";
}

/** @hidden */
function checkNotNull(arg: any, msg: string) {
  if (isNull(arg)) {
    throw new TypeError(`${msg} cannot be undefined or null`);
  }
}

/** @hidden */
function checkIsNumber(arg: any, msg: string) {
  if (typeof arg !== "number") {
    throw new TypeError(`${msg} must be a number, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsInteger(arg: any, msg: string) {
  checkIsNumber(arg, msg);
  if (!Number.isInteger(arg)) {
    throw new TypeError(`${msg} must be an integer, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsBoolean(arg: any, msg: string) {
  if (typeof arg !== "boolean") {
    throw new TypeError(`${msg} must be a boolean, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsString(arg: any, msg: string) {
  if (!isString(arg)) {
    throw new TypeError(`${msg} must be a string, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkOptionalString(arg: any, msg: string) {
  if (!(isString(arg) || isNull(arg))) {
    throw new TypeError(`${msg} must be a string?, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkMatchPattern(arg: any, pat: RegExp, msg: string) {
  checkIsString(arg, msg);
  if (!pat.test(arg)) {
    throw new Error(`${msg} is invalid pattern: ${arg}"`);
  }
}

/** @hidden */
function checkIsFunction(arg: any, msg: string) {
  if (typeof arg !== "function") {
    throw new TypeError(`${msg} must be a function, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsObject(arg: any, msg: string) {
  if (typeof arg !== "object") {
    throw new TypeError(`${msg} must be an object, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsUint8Array(arg: any, msg: string) {
  if (!(arg instanceof Uint8Array)) {
    throw new TypeError(`${msg} must be a Uint8Array, buf found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsOptions(arg: any, options: any[], msg: string) {
  if (!options.includes(arg)) {
    throw new RangeError(`${msg} is an invalid option: ${arg}`);
  }
}

/** @hidden */
function boundByIntegers(arg: any, bound: [number, number]) {
  if (arg < bound[0]) {
    return bound[0];
  }
  if (arg > bound[1]) {
    return bound[1];
  }
  return arg;
}

/** @hidden */
function setDefaultFields(arg: object, defaults: object) {
  for (const [key, val] of Object.entries(defaults)) {
    if (!Object.hasOwn(arg, key)) {
      Object.defineProperty(arg, key, { value: val, writable: true });
    }
  }
}

/**
 * The `Rsvim` global object.
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
  readonly buf = new RsvimBuf();
  readonly cmd = new RsvimCmd();
  readonly fs = new RsvimFs();
  readonly opt = new RsvimOpt();
  readonly rt = new RsvimRt();
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
   * located is binded to.
   *
   * :::warning
   * When the editor is not initialized, i.e. there's no buffer/window created. It
   * will return `undefined`. Once the editor is initialized, there will always have a
   * valid buffer binded to the "current" window (where your cursor is). It will return
   * the valid buffer ID.
   * :::
   *
   * @returns {(number | undefined)} It returns a valid buffer ID if the editor is initialized.
   * Otherwise it returns `undefined` if the editor is not initialized.
   *
   * @example
   * ```javascript
   * const bufId = Rsvim.buf.current();
   * ```
   */
  current(): number | undefined {
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
  list(): number[] {
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
  writeSync(bufId: number): number {
    checkIsInteger(bufId, `"Rsvim.buf.writeSync" bufId`);

    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.buf_write_sync(bufId);
  }
}

/**
 * The `Rsvim.cmd` global object for Ex commands.
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
   * Create a ex command with a callback function.
   *
   * :::warning
   * The builtin command `js` cannot be override.
   * :::
   *
   * @param {string} name - Command name that is going to create. Only letters (`a-z` and `A-Z`), digits (`0-9`), underscore (`_`) and exclamation (`!`) are allowed in a command name. Command name must not begin with a digit.
   * @param {RsvimCmd.CommandCallback} callback - Async callback function that implements the command. It accepts an `ctx` parameter that contains all the information when user is running it. See {@link RsvimCmd.CommandCallback}.
   * @param {RsvimCmd.CommandAttributes} attributes - (Optional) Attributes that control the command behavior, by default is `{bang:false, nargs:"0"}`, see {@link RsvimCmd.CommandAttributes}.
   * @param {RsvimCmd.CommandOptions} options - (Optional) Options that control how the command is created, by default is `{force:true}`, see {@link RsvimCmd.CommandOptions}.
   * @returns {(RsvimCmd.CommandDefinition | undefined)} It returns `undefined` is the command is newly created. Or it returns a command definition that was defined previously.
   *
   * @throws Throws {@link !TypeError} if any parameters are invalid. Or throws {@link Error} if command name or alias already exists, but `force` option is not set to override existing command forcibly.
   *
   * @example
   * ```javascript
   * async function write(ctx: any): void {
   *   try {
   *     const bytes = Rsvim.buf.writeSync(bufId);
   *
   *     // Call other async APIs
   *     const file = await Rsvim.fs.open("message.txt");
   *     const buffer = new Uint8Array(100);
   *     const read = await file.read(buffer);
   *     const message = new TextDecoder().decode(buffer);
   *
   *     Rsvim.cmd.echo(`Buffer ${bufId} has been saved, ${bytes} bytes written with message: ${message}`);
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
  ): RsvimCmd.CommandDefinition | undefined {
    checkMatchPattern(
      name,
      /^[A-Za-z_!][A-Za-z0-9_!]*$/,
      `"Rsvim.cmd.create" name`,
    );
    checkIsFunction(callback, `"Rsvim.cmd.create" callback`);

    attributes = attributes ?? { bang: false, nargs: "0" };
    checkIsObject(attributes, `"Rsvim.cmd.create" attributes`);
    setDefaultFields(attributes, { bang: false, nargs: "0" });
    checkIsBoolean(attributes.bang, `"Rsvim.cmd.create" bang attribute`);
    checkIsOptions(
      attributes.nargs,
      ["0", "1", "?", "+", "*"],
      `"Rsvim.cmd.create" nargs attribute`,
    );

    options = options ?? { force: true };
    checkIsObject(options, `"Rsvim.cmd.create" options`);
    setDefaultFields(options, { force: true });
    checkIsBoolean(options.force, `"Rsvim.cmd.create" force option`);
    checkOptionalString(options.alias, `"Rsvim.cmd.create" alias option`);

    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.cmd_create(
      name,
      callback,
      attributes,
      options,
    );
  }

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
  echo(message: any): void {
    checkNotNull(message, `"Rsvim.cmd.echo" message`);

    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.cmd_echo(message);
  }

  /**
   * List all registered ex command names.
   *
   * :::warning
   * The builtin `js` command will not be listed here.
   * :::
   *
   * @returns {string[]} Returns all registered ex command names, except the `js` command.
   *
   * @example
   * ```javascript
   * Rsvim.cmd.list().forEach((name) => {
   *   Rsvim.cmd.echo(`Command: ${name}`);
   * });
   * ```
   */
  list(): string[] {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.cmd_list();
  }

  /**
   * Get ex command definition by name.
   *
   * :::warning
   * The builtin `js` command cannot be get.
   * :::
   *
   * @returns {(RsvimCmd.CommandDefinition | undefined)} Returns command definition by its name, except the `js` command.
   *
   * @example
   * ```javascript
   * const def = Rsvim.cmd.get("write");
   * Rsvim.cmd.echo(`Command: ${def.name}`);
   * ```
   */
  get(name: string): RsvimCmd.CommandDefinition | undefined {
    checkIsString(name, `"Rsvim.cmd.get" name`);

    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.cmd_get(name);
  }

  /**
   * Remove an ex command by name.
   *
   * :::warning
   * The builtin command `js` cannot be removed.
   * :::
   *
   * @param {string} name - The command name to be removed.
   * @returns {(RsvimCmd.CommandDefinition | undefined)} Returns the removed {@link RsvimCmd.CommandDefinition}, or `undefined` if no command is been removed.
   *
   * @throws Throws {@link !TypeError} if name is not a string.
   *
   * @example
   * ```javascript
   * Rsvim.cmd.list().forEach((cmd) => {
   *   // Remove all registered commands.
   *   Rsvim.cmd.remove(cmd.name);
   * });
   * ```
   */
  remove(name: string): RsvimCmd.CommandDefinition | undefined {
    checkIsString(name, `"Rsvim.cmd.remove" name`);

    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.cmd_remove(name);
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
     * @defaultValue `false`
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
     * @defaultValue `0`
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
     * @defaultValue `true`
     */
    force?: boolean;

    /**
     * Command alias, i.e. short name.
     *
     * For example, the `w` is alias for `write`.
     *
     * @defaultValue `undefined`
     */
    alias?: string;
  };

  /**
   * Command callback function, this is the backend logic that implements a user ex command.
   *
   * @see {@link RsvimCmd.create}
,  */
  export type CommandCallback = (ctx: any) => Promise<void>;

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
 * The `Rsvim.fs` global object for file system and file IO.
 *
 * @example
 * ```javascript
 * // Create a alias to 'Rsvim.fs'.
 * const fs = Rsvim.fs;
 * ```
 *
 * @category General APIs
 * @hideconstructor
 */
export class RsvimFs {
  /**
   * Open a file and resolve to an instance of {@link RsvimFs.File}. The file does not need to previously exist if using the `create` or `createNew` open options.
   * The caller have to close the file to prevent resource leaking, see {@link RsvimFs.File.close}.
   *
   * @param {string} path - File path.
   * @param {RsvimFs.OpenOptions} options - (Optional) Open options, by default is `{read: true}`. See {@link RsvimFs.OpenOptions}.
   * @returns {Promise<RsvimFs.File>} It resolves to an instance of {@link RsvimFs.File}.
   *
   * @throws Throws {@link !TypeError} if any parameters are invalid. Or throws {@link Error} if failed to open/create the file.
   *
   * @example
   * ```javascript
   * const file = await Rsvim.fs.open("README.md");
   * ```
   */
  open(path: string, options?: RsvimFs.OpenOptions): Promise<RsvimFs.File> {
    checkIsString(path, `"Rsvim.fs.open" path`);

    options = options ?? { read: true };
    checkIsObject(options, `"Rsvim.fs.open" options`);
    setDefaultFields(options, {
      append: false,
      create: false,
      createNew: false,
      read: false,
      truncate: false,
      write: false,
    });
    checkIsBoolean(options.append, `"Rsvim.fs.open" append option`);
    checkIsBoolean(options.create, `"Rsvim.fs.open" create option`);
    checkIsBoolean(options.createNew, `"Rsvim.fs.open" createNew option`);
    checkIsBoolean(options.read, `"Rsvim.fs.open" read option`);
    checkIsBoolean(options.truncate, `"Rsvim.fs.open" truncate option`);
    checkIsBoolean(options.write, `"Rsvim.fs.open" write option`);

    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject
      .fs_open(path, options)
      .then((handle: any) => new RsvimFs.File(handle));
  }

  /**
   * The sync version of {@link open}.
   *
   * @param {string} path
   * @param {RsvimFs.OpenOptions} options
   * @returns {RsvimFs.File}
   *
   * @throws
   *
   * @example
   * ```javascript
   * const file = Rsvim.fs.openSync("README.md");
   * ```
   */
  openSync(path: string, options?: RsvimFs.OpenOptions): RsvimFs.File {
    checkIsString(path, `"Rsvim.fs.openSync" path`);

    options = options ?? { read: true };
    checkIsObject(options, `"Rsvim.fs.open" options`);
    setDefaultFields(options, {
      append: false,
      create: false,
      createNew: false,
      read: false,
      truncate: false,
      write: false,
    });
    checkIsBoolean(options.append, `"Rsvim.fs.open" append option`);
    checkIsBoolean(options.create, `"Rsvim.fs.open" create option`);
    checkIsBoolean(options.createNew, `"Rsvim.fs.open" createNew option`);
    checkIsBoolean(options.read, `"Rsvim.fs.open" read option`);
    checkIsBoolean(options.truncate, `"Rsvim.fs.open" truncate option`);
    checkIsBoolean(options.write, `"Rsvim.fs.open" write option`);

    // @ts-ignore Ignore warning
    const handle = __InternalRsvimGlobalObject.fs_open_sync(path, options);
    return new RsvimFs.File(handle);
  }
}

export namespace RsvimFs {
  /**
   * Open options.
   *
   * :::tip
   * It is same with [std::fs::OpenOptions](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html) in rust std library.
   * :::
   *
   * @see {@link RsvimFs.open}
   */
  export type OpenOptions = {
    /**
     * Set the file for append mode.
     *
     * @defaultValue `false`
,    */
    append?: boolean;

    /**
     * Create a new file or open it if it already exists.
     *
     * In order for the file to be created, `write` or `append` access must be used.
     *
     * @defaultValue `false`
,    */
    create?: boolean;

    /**
     * Create a new file, failing if it already exists.
     *
     * If this option is set, `create` and `truncate` options are ignored.
     *
     * @defaultValue `false`
,    */
    createNew?: boolean;

    /**
     * Set the file for read access.
     *
     * @defaultValue `false`
,    */
    read?: boolean;

    /**
     * Open the file and truncate the file to `0` length if it already exists.
     *
     * @defaultValue `false`
,    */
    truncate?: boolean;

    /**
     * Set the file for write access. If the file already exists, any "write" calls on it will 
     * overwrite the contents, without truncating it.
     *
     * @defaultValue `false`
,    */
    write?: boolean;
  };

  /**
   * The File object that access to an open file on filesystem.
   *
   * @see {@link RsvimFs.open}
   */
  export class File {
    /** @hidden */
    #handle: any;

    /** @hidden */
    constructor(handle: any) {
      this.#handle = handle;
    }

    /**
     * Close the file.
     *
     * @example
     * ```javascript
     * const file = await Rsvim.fs.open("README.md");
     * // do work with the `file` object
     * file.close();
     *
     * // Or
     *
     * using file = await Rsvim.fs.open("README.md");
     * // do work with the `file` object
     * ```
     */
    close(): void {
      if (!isNull(this.#handle)) {
        // @ts-ignore Ignore warning
        __InternalRsvimGlobalObject.fs_close(this.#handle);
      }
      this.#handle = null;
    }

    /**
     * Close the file with `using` without `close` API.
     *
     * @example
     * ```javascript
     * using file = await Rsvim.fs.open("README.md");
     * // do work with the `file` object
     * ```
     *
     * @see {@link close}
     */
    [Symbol.dispose](): void {
      this.close();
    }

    /**
     * File is already been closed.
     *
     * @example
     * ```javascript
     * const file = await Rsvim.fs.open("README.md");
     * if (!file.isDisposed()) {
     *   file.close();
     * }
     * ```
     */
    get isDisposed(): boolean {
      return isNull(this.#handle);
    }

    /**
     * Read a file into a buffer.
     *
     * :::warning
     * It is not guaranteed that the full buffer will be read in a single call.
     * :::
     *
     * @param {Uint8Array} buf - Read bytes into buffer.
     * @returns {Promise<number>} It resolves to either the number of bytes read during the operation or `0`(EOF) if there was no more to read.
     *
     * @throws Throws {@link !TypeError} if buf is not a Uint8Array.
     *
     * @example
     * ```javascript
     * using file = await Rsvim.fs.open("README.md");
     * const buf = new Uint8Array(100);
     * const n = await file.read(buf); // read 11 bytes
     * const text = new TextDecoder().decode(buf); // decode into UTF-8 string "hello world"
     * ```
     */
    read(buf: Uint8Array): Promise<number> {
      checkIsUint8Array(buf, `"RsvimFs.File.read" buf`);

      // @ts-ignore Ignore warning
      return __InternalRsvimGlobalObject.fs_read(this.#handle, buf.buffer);
    }

    /**
     * Sync version of {@link read}.
     *
     * @param {Uint8Array} buf
     * @returns {number}
     *
     * @throws
     *
     * @example
     * ```javascript
     * using file = await Rsvim.fs.open("README.md");
     * const buf = new Uint8Array(100);
     * const n = file.readSync(buf); // read 11 bytes
     * const text = new TextDecoder().decode(buf); // decode into UTF-8 string "hello world"
     * ```
     */
    readSync(buf: Uint8Array): number {
      checkIsUint8Array(buf, `"RsvimFs.File.readSync" buf`);

      // @ts-ignore Ignore warning
      return __InternalRsvimGlobalObject.fs_read_sync(this.#handle, buf.buffer);
    }

    /**
     * Write a buffer into a file.
     *
     * :::warning
     * It is not guaranteed that the full buffer will be written in a single call.
     * :::
     *
     * @param {Uint8Array} buf - Read bytes into buffer.
     * @returns {Promise<number>} It resolves to either the number of bytes written during the operation or `0` if there was nothing to write.
     *
     * @throws Throws {@link !TypeError} if buf is not a Uint8Array.
     *
     * @example
     * ```javascript
     * using file = await Rsvim.fs.open("README.md", {write:true,create:true});
     * const buf = new TextEncoder().encode("hello world");
     * const n = await file.write(buf); // write 11 bytes
     * ```
     */
    write(buf: Uint8Array): Promise<number> {
      checkIsUint8Array(buf, `"RsvimFs.File.write" buf`);

      // @ts-ignore Ignore warning
      return __InternalRsvimGlobalObject.fs_write(this.#handle, buf.buffer);
    }

    /**
     * Sync version of {@link write}.
     *
     * @param {Uint8Array} buf
     * @returns {number}
     *
     * @throws
     *
     * @example
     * ```javascript
     * using file = await Rsvim.fs.open("README.md", {write:true,create:true});
     * const buf = new TextEncoder().encode("hello world");
     * const n = file.writeSync(buf); // write 11 bytes
     * ```
     */
    writeSync(buf: Uint8Array): number {
      checkIsUint8Array(buf, `"RsvimFs.File.writeSync" buf`);

      // @ts-ignore Ignore warning
      return __InternalRsvimGlobalObject.fs_write_sync(
        this.#handle,
        buf.buffer,
      );
    }
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
  get expandTab(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_expand_tab();
  }

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
  set expandTab(value: boolean) {
    checkIsBoolean(value, `"Rsvim.opt.expandTab" value`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_expand_tab(value);
  }

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
   * @returns {RsvimOpt.FileEncodingOption}
   *
   * @defaultValue `"utf-8"`
   *
   * @example
   * ```javascript
   * // Get the 'file-encoding' option.
   * const value = Rsvim.opt.fileEncoding;
   * ```
   */
  get fileEncoding(): RsvimOpt.FileEncodingOption {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_file_encoding();
  }

  /**
   * Set the _file-encoding_ option.
   *
   * @param {RsvimOpt.FileEncodingOption} value - The _file-encoding_ option.
   * @throws Throws {@link !RangeError} if value is an invalid option.
   *
   * @example
   * ```javascript
   * // Set the 'file-encoding' option.
   * Rsvim.opt.fileEncoding = "utf-8";
   * ```
   */
  set fileEncoding(value: RsvimOpt.FileEncodingOption) {
    checkIsOptions(value, ["utf-8"], `"Rsvim.opt.fileEncoding" value`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_file_encoding(value);
  }

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
   * @returns {RsvimOpt.FileFormatOption}
   *
   * @defaultValue `"dos"` for Windows/MS-DOS, `"unix"` for Linux/Unix/MacOS.
   *
   * @example
   * ```javascript
   * // Get the 'file-format' option.
   * const value = Rsvim.opt.fileFormat;
   * ```
   */
  get fileFormat(): RsvimOpt.FileFormatOption {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_file_format();
  }

  /**
   * Set the _file-format_ option.
   *
   * @param {RsvimOpt.FileFormatOption} value - The _file-format_ option.
   * @throws Throws {@link !RangeError} if value is an invalid option.
   *
   * @example
   * ```javascript
   * // Set the 'file-format' option.
   * Rsvim.opt.fileFormat = "unix";
   * ```
   */
  set fileFormat(value: RsvimOpt.FileFormatOption) {
    checkIsOptions(
      value,
      ["dos", "unix", "mac"],
      `"Rsvim.opt.fileFormat" value`,
    );
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_file_format(value);
  }

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
  get lineBreak(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_line_break();
  }

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
  set lineBreak(value: boolean) {
    checkIsBoolean(value, `"Rsvim.opt.lineBreak" value`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_line_break(value);
  }

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
  get shiftWidth(): number {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_shift_width();
  }

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
  set shiftWidth(value: number) {
    checkIsInteger(value, `"Rsvim.opt.shiftWidth" value`);
    value = boundByIntegers(value, [1, 255]);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_shift_width(value);
  }

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
  get tabStop(): number {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_tab_stop();
  }

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
  set tabStop(value: number) {
    checkIsInteger(value, `"Rsvim.opt.tabStop" value`);
    value = boundByIntegers(value, [1, 255]);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_tab_stop(value);
  }

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
  get wrap(): boolean {
    // @ts-ignore Ignore warning
    return __InternalRsvimGlobalObject.opt_get_wrap();
  }

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
  set wrap(value: boolean) {
    checkIsBoolean(value, `"Rsvim.opt.wrap" value`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.opt_set_wrap(value);
  }
}

export namespace RsvimOpt {
  /**
   * @inline
   */
  export type FileEncodingOption = "utf-8";

  /**
   * @inline
   */
  export type FileFormatOption = "dos" | "unix" | "mac";
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
   * @param {exitCode?} exitCode - (Optional) The editor process exit with this exit code, by default with code `0`.
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
  exit(exitCode?: number): void {
    exitCode = exitCode ?? 0;
    checkIsInteger(exitCode, `"Rsvim.rt.exit" code`);
    // @ts-ignore Ignore warning
    __InternalRsvimGlobalObject.rt_exit(exitCode);
  }
}

(function (globalThis: { Rsvim: Rsvim }) {
  globalThis.Rsvim = new Rsvim();
})(globalThis as unknown as { Rsvim: Rsvim });
