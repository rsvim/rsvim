export class Rsvim {
    buf = new RsvimBuf();
    cmd = new RsvimCmd();
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
        if (typeof bufId !== "number") {
            throw new Error(`"Rsvim.buf.write" bufId parameter must be a integer value, but found ${bufId} (${typeof bufId})`);
        }
        return __InternalRsvimGlobalObject.buf_write_sync(bufId);
    }
}
export class RsvimCmd {
    echo(message) {
        if (message === undefined || message === null) {
            throw new Error('"Rsvim.cmd.echo" message parameter cannot be undefined or null');
        }
        __InternalRsvimGlobalObject.cmd_echo(message);
    }
}
export class RsvimOpt {
    get fileEncoding() {
        return __InternalRsvimGlobalObject.opt_get_file_encoding();
    }
    set fileEncoding(value) {
        if (value !== "utf-8") {
            throw new Error(`"Rsvim.opt.fileEncoding" parameter must be a valid option, but found ${value} (${typeof value})`);
        }
        __InternalRsvimGlobalObject.opt_set_file_encoding(value);
    }
    get fileFormat() {
        return __InternalRsvimGlobalObject.opt_get_file_format();
    }
    set fileFormat(value) {
        if (value !== "dos" && value !== "unix" && value !== "mac") {
            throw new Error(`"Rsvim.opt.fileFormat" parameter must be a valid option, but found ${value} (${typeof value})`);
        }
        __InternalRsvimGlobalObject.opt_set_file_format(value);
    }
    get lineBreak() {
        return __InternalRsvimGlobalObject.opt_get_line_break();
    }
    set lineBreak(value) {
        if (typeof value !== "boolean") {
            throw new Error(`"Rsvim.opt.lineBreak" must be a boolean value, but found ${value} (${typeof value})`);
        }
        __InternalRsvimGlobalObject.opt_set_line_break(value);
    }
    get tabStop() {
        return __InternalRsvimGlobalObject.opt_get_tab_stop();
    }
    set tabStop(value) {
        if (typeof value !== "number" || value < 1 || value > 65535) {
            throw new Error(`"Rsvim.opt.tabStop" parameter must be an integer value between [1,65535], but found ${value} (${typeof value})`);
        }
        __InternalRsvimGlobalObject.opt_set_tab_stop(value);
    }
    get wrap() {
        return __InternalRsvimGlobalObject.opt_get_wrap();
    }
    set wrap(value) {
        if (typeof value !== "boolean") {
            throw new Error(`"Rsvim.opt.wrap" must be a boolean value, but found ${value} (${typeof value})`);
        }
        __InternalRsvimGlobalObject.opt_set_wrap(value);
    }
}
export class RsvimRt {
    exit(exitCode) {
        if (exitCode !== undefined && typeof exitCode !== "number") {
            throw new Error('"Rsvim.rt.exit" exit code parameter must be a valid integer or undefined');
        }
        __InternalRsvimGlobalObject.rt_exit(exitCode);
    }
}
(function (globalThis) {
    globalThis.Rsvim = new Rsvim();
})(globalThis);
