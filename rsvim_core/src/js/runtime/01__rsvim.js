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
        return __InternalRsvimGlobalObject.cmd_echo(message);
    }
}
export class RsvimOpt {
    get lineBreak() {
        return __InternalRsvimGlobalObject.opt_get_line_break();
    }
    set lineBreak(value) {
        if (typeof value !== "boolean") {
            throw new Error(`"Rsvim.opt.lineBreak" must be a boolean value, but found ${value} (${typeof value})`);
        }
        __InternalRsvimGlobalObject.opt_set_line_break(value);
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
        return __InternalRsvimGlobalObject.rt_exit(exitCode);
    }
}
(function (globalThis) {
    globalThis.Rsvim = new Rsvim();
})(globalThis);
