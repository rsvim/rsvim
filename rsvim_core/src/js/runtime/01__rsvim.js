export class Rsvim {
    opt = new RsvimOpt();
}
export class RsvimOpt {
    get wrap() {
        return __InternalRsvimGlobalObject.opt_get_wrap();
    }
    set wrap(value) {
        if (typeof value !== "boolean") {
            throw new Error(`"Rsvim.opt.wrap" must be a boolean value, but found ${value} (${typeof value})`);
        }
        __InternalRsvimGlobalObject.opt_set_wrap(value);
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
}
(function (globalThis) {
    globalThis.Rsvim = new Rsvim();
})(globalThis);
