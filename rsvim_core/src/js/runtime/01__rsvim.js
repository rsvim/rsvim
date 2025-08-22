export class Rsvim {
    buf = new RsvimBuf();
    cmd = new RsvimCmd();
    opt = new RsvimOpt();
}
export class RsvimBuf {
    currentBuffer() {
        return __InternalRsvimGlobalObject.buf_current_buffer();
    }
    listAllBuffers() {
        return __InternalRsvimGlobalObject.buf_list_all_buffers();
    }
    write(bufId) {
        if (typeof bufId !== "number") {
            throw new Error(`"Rsvim.buf.write" bufId parameter must be a integer value, but found ${bufId} (${typeof bufId})`);
        }
        __InternalRsvimGlobalObject.buf_write(bufId);
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
(function (globalThis) {
    globalThis.Rsvim = new Rsvim();
})(globalThis);
