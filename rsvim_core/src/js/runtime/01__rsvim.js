function checkNotNull(arg, msg) {
    if (arg === undefined || arg === null) {
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
    if (typeof arg !== "string") {
        throw new TypeError(`${msg} must be a string, but found ${typeof arg}`);
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
        throw new RangeError(`${msg} is invalid option: ${arg}`);
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
        if (attributes === undefined || attributes === null) {
            attributes = {};
        }
        checkIsObject(attributes, `"Rsvim.cmd.create" attributes`);
        if (!Object.hasOwn(attributes, "bang")) {
            attributes.bang = false;
        }
        if (!Object.hasOwn(attributes, "nargs")) {
            attributes.nargs = "0";
        }
        checkIsBoolean(attributes.bang, `"Rsvim.cmd.create" attributes.bang`);
        checkIsOptions(attributes.nargs, ["0", "1", "?", "+", "*"], `"Rsvim.cmd.create" attributes.nargs`);
        if (options === undefined || options === null) {
            options = {};
        }
        checkIsObject(options, `"Rsvim.cmd.create" options`);
        if (!Object.hasOwn(options, "force")) {
            options.force = true;
        }
        checkIsBoolean(options.force, `"Rsvim.cmd.create" options.force`);
        if (options.alias !== undefined) {
            checkIsString(options.alias, `"Rsvim.cmd.create" options.alias`);
        }
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
        if (options === undefined || options === null) {
            options = { read: true };
        }
        checkIsObject(options, `"Rsvim.fs.open" options`);
        if (!Object.hasOwn(options, "append")) {
            options.append = false;
        }
        if (!Object.hasOwn(options, "create")) {
            options.create = false;
        }
        if (!Object.hasOwn(options, "createNew")) {
            options.createNew = false;
        }
        if (!Object.hasOwn(options, "read")) {
            options.read = false;
        }
        if (!Object.hasOwn(options, "truncate")) {
            options.truncate = false;
        }
        if (!Object.hasOwn(options, "write")) {
            options.write = false;
        }
        return __InternalRsvimGlobalObject
            .fs_open(path, options)
            .then((handle) => new RsvimFs.File(handle));
    }
    openSync(path, options) {
        checkIsString(path, `"Rsvim.fs.openSync" path`);
        if (options === undefined || options === null) {
            options = { read: true };
        }
        checkIsObject(options, `"Rsvim.fs.openSync" options`);
        if (!Object.hasOwn(options, "append")) {
            options.append = false;
        }
        if (!Object.hasOwn(options, "create")) {
            options.create = false;
        }
        if (!Object.hasOwn(options, "createNew")) {
            options.createNew = false;
        }
        if (!Object.hasOwn(options, "read")) {
            options.read = false;
        }
        if (!Object.hasOwn(options, "truncate")) {
            options.truncate = false;
        }
        if (!Object.hasOwn(options, "write")) {
            options.write = false;
        }
        const handle = __InternalRsvimGlobalObject.fs_open_sync(path, options);
        return new RsvimFs.File(handle);
    }
}
(function (RsvimFs) {
    class File {
        #__handle;
        constructor(handle) {
            this.#__handle = handle;
        }
        close() {
            __InternalRsvimGlobalObject.fs_close(this.#__handle);
        }
        isClosed() {
            return __InternalRsvimGlobalObject.fs_is_closed(this.#__handle);
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
        if (exitCode === undefined || exitCode === null) {
            exitCode = 0;
        }
        checkIsInteger(exitCode, `"Rsvim.rt.exit" exit code`);
        __InternalRsvimGlobalObject.rt_exit(exitCode);
    }
}
(function (globalThis) {
    globalThis.Rsvim = new Rsvim();
})(globalThis);
