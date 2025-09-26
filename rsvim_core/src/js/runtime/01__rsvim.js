class RsvimImpl {
    buf = new RsvimBufImpl();
    cmd = new RsvimCmdImpl();
    opt = new RsvimOptImpl();
    rt = new RsvimRtImpl();
}
function checkNotNull(arg, msg) {
    if (arg === undefined || arg === null) {
        throw new TypeError(`${msg} cannot be undefined or null`);
    }
}
function checkIsNumber(arg, msg) {
    if (typeof arg !== "number") {
        throw new TypeError(`${msg} must be an integer, but found ${typeof arg}`);
    }
}
function checkIsInteger(arg, msg) {
    checkIsNumber(arg, msg);
    if (!Number.isInteger(arg)) {
        throw new TypeError(`${msg} must be an integer, but found ${typeof arg}`);
    }
}
function checkIsOptionalInteger(arg, msg) {
    if (arg !== undefined && typeof arg !== "number") {
        throw new TypeError(`${msg} must be an integer or undefined, but found ${typeof arg}`);
    }
    checkIsInteger(arg, msg);
}
function checkIsBoolean(arg, msg) {
    if (typeof arg !== "boolean") {
        throw new TypeError(`${msg} must be a boolean, but found ${typeof arg}`);
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
class RsvimBufImpl {
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
class RsvimCmdImpl {
    create(name, callback, attributes, options) {
        return undefined;
    }
    echo(message) {
        checkNotNull(message, `"Rsvim.cmd.echo" message`);
        __InternalRsvimGlobalObject.cmd_echo(message);
    }
}
class RsvimOptImpl {
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
class RsvimRtImpl {
    exit(exitCode) {
        checkIsOptionalInteger(exitCode, `"Rsvim.rt.exit" exit code`);
        __InternalRsvimGlobalObject.rt_exit(exitCode);
    }
}
(function (globalThis) {
    globalThis.Rsvim = new RsvimImpl();
})(globalThis);
export {};
