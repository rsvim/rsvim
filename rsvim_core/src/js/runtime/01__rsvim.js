function isNull(arg) {
    return arg === undefined || arg === null;
}
function isString(arg) {
    return typeof arg === "string";
}
function checkNotNull(arg, msg) {
    if (isNull(arg)) {
        throw new TypeError(`${msg} cannot be undefined or null`);
    }
}
function checkIsNumber(arg, msg) {
    if (typeof arg !== "number") {
        throw new TypeError(`${msg} must be a number, but found ${typeof arg}`);
    }
}
function checkIsInteger(arg, msg) {
    checkIsNumber(arg, msg);
    if (!Number.isInteger(arg)) {
        throw new TypeError(`${msg} must be an integer, but found ${typeof arg}`);
    }
}
function checkIsBoolean(arg, msg) {
    if (typeof arg !== "boolean") {
        throw new TypeError(`${msg} must be a boolean, but found ${typeof arg}`);
    }
}
function checkIsString(arg, msg) {
    if (!isString(arg)) {
        throw new TypeError(`${msg} must be a string, but found ${typeof arg}`);
    }
}
function checkOptionalString(arg, msg) {
    if (!(isString(arg) || isNull(arg))) {
        throw new TypeError(`${msg} must be a string?, but found ${typeof arg}`);
    }
}
function checkMatchPattern(arg, pat, msg) {
    checkIsString(arg, msg);
    if (!pat.test(arg)) {
        throw new Error(`${msg} is invalid pattern: ${arg}"`);
    }
}
function checkIsFunction(arg, msg) {
    if (typeof arg !== "function") {
        throw new TypeError(`${msg} must be a function, but found ${typeof arg}`);
    }
}
function checkIsObject(arg, msg) {
    if (typeof arg !== "object") {
        throw new TypeError(`${msg} must be an object, but found ${typeof arg}`);
    }
}
function checkIsOptions(arg, options, msg) {
    if (!options.includes(arg)) {
        throw new RangeError(`${msg} is an invalid option: ${arg}`);
    }
}
function boundByIntegers(arg, bound) {
    if (arg < bound[0]) {
        return bound[0];
    }
    if (arg > bound[1]) {
        return bound[1];
    }
    return arg;
}
function setDefaultFields(arg, defaults) {
    for (const [key, val] of Object.entries(defaults)) {
        if (!Object.hasOwn(arg, key)) {
            Object.defineProperty(arg, key, { value: val, writable: true });
        }
    }
}
export class Rsvim {
    buf = new RsvimBuf();
    cmd = new RsvimCmd();
    fs = new RsvimFs();
    opt = new RsvimOpt();
    rt = new RsvimRt();
}
export class RsvimBuf {
    current() {
        return __InternalRsvimGlobalObject.buf_current();
    }
    list() {
        return __InternalRsvimGlobalObject.buf_list();
    }
    writeSync(bufId) {
        checkIsInteger(bufId, `"Rsvim.buf.writeSync" bufId`);
        return __InternalRsvimGlobalObject.buf_write_sync(bufId);
    }
}
export class RsvimCmd {
    create(name, callback, attributes, options) {
        checkMatchPattern(name, /^[A-Za-z_!][A-Za-z0-9_!]*$/, `"Rsvim.cmd.create" name`);
        checkIsFunction(callback, `"Rsvim.cmd.create" callback`);
        attributes = attributes ?? { bang: false, nargs: "0" };
        checkIsObject(attributes, `"Rsvim.cmd.create" attributes`);
        setDefaultFields(attributes, { bang: false, nargs: "0" });
        checkIsBoolean(attributes.bang, `"Rsvim.cmd.create" bang attribute`);
        checkIsOptions(attributes.nargs, ["0", "1", "?", "+", "*"], `"Rsvim.cmd.create" nargs attribute`);
        options = options ?? { force: true };
        checkIsObject(options, `"Rsvim.cmd.create" options`);
        setDefaultFields(options, { force: true });
        checkIsBoolean(options.force, `"Rsvim.cmd.create" force option`);
        checkOptionalString(options.alias, `"Rsvim.cmd.create" alias option`);
        return __InternalRsvimGlobalObject.cmd_create(name, callback, attributes, options);
    }
    echo(message) {
        checkNotNull(message, `"Rsvim.cmd.echo" message`);
        __InternalRsvimGlobalObject.cmd_echo(message);
    }
    list() {
        return __InternalRsvimGlobalObject.cmd_list();
    }
    get(name) {
        checkIsString(name, `"Rsvim.cmd.get" name`);
        return __InternalRsvimGlobalObject.cmd_get(name);
    }
    remove(name) {
        checkIsString(name, `"Rsvim.cmd.remove" name`);
        return __InternalRsvimGlobalObject.cmd_remove(name);
    }
}
export class RsvimFs {
    open(path, options) {
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
        return __InternalRsvimGlobalObject
            .fs_open(path, options)
            .then((handle) => new RsvimFs.File(handle));
    }
    openSync(path, options) {
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
        const handle = __InternalRsvimGlobalObject.fs_open_sync(path, options);
        return new RsvimFs.File(handle);
    }
}
(function (RsvimFs) {
    class File {
        #handle;
        constructor(handle) {
            this.#handle = handle;
        }
        close() {
            if (!isNull(this.#handle)) {
                __InternalRsvimGlobalObject.fs_close(this.#handle);
            }
            this.#handle = null;
        }
        [Symbol.dispose]() {
            this.close();
        }
        get isDisposed() {
            return isNull(this.#handle);
        }
    }
    RsvimFs.File = File;
})(RsvimFs || (RsvimFs = {}));
export class RsvimOpt {
    get expandTab() {
        return __InternalRsvimGlobalObject.opt_get_expand_tab();
    }
    set expandTab(value) {
        checkIsBoolean(value, `"Rsvim.opt.expandTab" value`);
        __InternalRsvimGlobalObject.opt_set_expand_tab(value);
    }
    get fileEncoding() {
        return __InternalRsvimGlobalObject.opt_get_file_encoding();
    }
    set fileEncoding(value) {
        checkIsOptions(value, ["utf-8"], `"Rsvim.opt.fileEncoding" value`);
        __InternalRsvimGlobalObject.opt_set_file_encoding(value);
    }
    get fileFormat() {
        return __InternalRsvimGlobalObject.opt_get_file_format();
    }
    set fileFormat(value) {
        checkIsOptions(value, ["dos", "unix", "mac"], `"Rsvim.opt.fileFormat" value`);
        __InternalRsvimGlobalObject.opt_set_file_format(value);
    }
    get lineBreak() {
        return __InternalRsvimGlobalObject.opt_get_line_break();
    }
    set lineBreak(value) {
        checkIsBoolean(value, `"Rsvim.opt.lineBreak" value`);
        __InternalRsvimGlobalObject.opt_set_line_break(value);
    }
    get shiftWidth() {
        return __InternalRsvimGlobalObject.opt_get_shift_width();
    }
    set shiftWidth(value) {
        checkIsInteger(value, `"Rsvim.opt.shiftWidth" value`);
        value = boundByIntegers(value, [1, 255]);
        __InternalRsvimGlobalObject.opt_set_shift_width(value);
    }
    get tabStop() {
        return __InternalRsvimGlobalObject.opt_get_tab_stop();
    }
    set tabStop(value) {
        checkIsInteger(value, `"Rsvim.opt.tabStop" value`);
        value = boundByIntegers(value, [1, 255]);
        __InternalRsvimGlobalObject.opt_set_tab_stop(value);
    }
    get wrap() {
        return __InternalRsvimGlobalObject.opt_get_wrap();
    }
    set wrap(value) {
        checkIsBoolean(value, `"Rsvim.opt.wrap" value`);
        __InternalRsvimGlobalObject.opt_set_wrap(value);
    }
}
export class RsvimRt {
    exit(exitCode) {
        exitCode = exitCode ?? 0;
        checkIsInteger(exitCode, `"Rsvim.rt.exit" code`);
        __InternalRsvimGlobalObject.rt_exit(exitCode);
    }
}
(function (globalThis) {
    globalThis.Rsvim = new Rsvim();
})(globalThis);
