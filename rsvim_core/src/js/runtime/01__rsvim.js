class RsvimImpl {
    buf = new RsvimBufImpl();
    cmd = new RsvimCmdImpl();
    opt = new RsvimOptImpl();
    rt = new RsvimRtImpl();
}
class RsvimBufImpl {
    current() {
        return __InternalRsvimGlobalObject.buf_current();
    }
    list() {
        return __InternalRsvimGlobalObject.buf_list();
    }
    writeSync(bufId) {
        if (typeof bufId !== "number") {
            throw new TypeError(`"Rsvim.buf.write" bufId must be an integer, but found ${typeof bufId}`);
        }
        return __InternalRsvimGlobalObject.buf_write_sync(bufId);
    }
}
class RsvimCmdImpl {
    create(name, callback, attr, opts) {
        return undefined;
    }
    echo(message) {
        if (message === undefined || message === null) {
            throw new TypeError('"Rsvim.cmd.echo" message cannot be undefined or null');
        }
        __InternalRsvimGlobalObject.cmd_echo(message);
    }
}
class RsvimOptImpl {
    get expandTab() {
        return __InternalRsvimGlobalObject.opt_get_expand_tab();
    }
    set expandTab(value) {
        if (typeof value !== "boolean") {
            throw new TypeError(`"Rsvim.opt.expandTab" parameter must be a boolean, but found ${typeof value}`);
        }
        __InternalRsvimGlobalObject.opt_set_expand_tab(value);
    }
    get fileEncoding() {
        return __InternalRsvimGlobalObject.opt_get_file_encoding();
    }
    set fileEncoding(value) {
        if (value !== "utf-8") {
            throw new Error(`"Rsvim.opt.fileEncoding" parameter is invalid: ${value}`);
        }
        __InternalRsvimGlobalObject.opt_set_file_encoding(value);
    }
    get fileFormat() {
        return __InternalRsvimGlobalObject.opt_get_file_format();
    }
    set fileFormat(value) {
        if (value !== "dos" && value !== "unix" && value !== "mac") {
            throw new Error(`"Rsvim.opt.fileFormat" parameter is invalid: ${value}`);
        }
        __InternalRsvimGlobalObject.opt_set_file_format(value);
    }
    get lineBreak() {
        return __InternalRsvimGlobalObject.opt_get_line_break();
    }
    set lineBreak(value) {
        if (typeof value !== "boolean") {
            throw new TypeError(`"Rsvim.opt.lineBreak" parameter must be a boolean, but found ${typeof value}`);
        }
        __InternalRsvimGlobalObject.opt_set_line_break(value);
    }
    get shiftWidth() {
        return __InternalRsvimGlobalObject.opt_get_shift_width();
    }
    set shiftWidth(value) {
        if (typeof value !== "number") {
            throw new TypeError(`"Rsvim.opt.shiftWidth" parameter must be an integer, but found ${typeof value}`);
        }
        if (value < 1 || value > 255) {
            throw new RangeError(`"Rsvim.opt.shiftWidth" parameter must be between [1,255], but found ${value}`);
        }
        __InternalRsvimGlobalObject.opt_set_shift_width(value);
    }
    get tabStop() {
        return __InternalRsvimGlobalObject.opt_get_tab_stop();
    }
    set tabStop(value) {
        if (typeof value !== "number") {
            throw new TypeError(`"Rsvim.opt.tabStop" parameter must be an integer, but found ${typeof value}`);
        }
        if (value < 1 || value > 255) {
            throw new RangeError(`"Rsvim.opt.tabStop" parameter must be between [1,255], but found ${value}`);
        }
        __InternalRsvimGlobalObject.opt_set_tab_stop(value);
    }
    get wrap() {
        return __InternalRsvimGlobalObject.opt_get_wrap();
    }
    set wrap(value) {
        if (typeof value !== "boolean") {
            throw new TypeError(`"Rsvim.opt.wrap" parameter must be a boolean, but found ${typeof value}`);
        }
        __InternalRsvimGlobalObject.opt_set_wrap(value);
    }
}
class RsvimRtImpl {
    exit(exitCode) {
        if (exitCode !== undefined && typeof exitCode !== "number") {
            throw new Error('"Rsvim.rt.exit" exit code parameter must be a valid integer or undefined');
        }
        __InternalRsvimGlobalObject.rt_exit(exitCode);
    }
}
(function (globalThis) {
    globalThis.Rsvim = new RsvimImpl();
})(globalThis);
export {};
