import infra from "rsvim:ext/infra";
var Rsvim = (function () {
    function Rsvim() {
        this.opt = new RsvimOpt();
    }
    return Rsvim;
}());
export { Rsvim };
var RsvimOpt = (function () {
    function RsvimOpt() {
    }
    Object.defineProperty(RsvimOpt.prototype, "wrap", {
        get: function () {
            return __InternalRsvimGlobalObject.opt_get_wrap();
        },
        set: function (value) {
            if (typeof value !== "boolean") {
                throw new Error("\"Rsvim.opt.wrap\" value must be boolean type, but found ".concat(infra.stringify(value)));
            }
            __InternalRsvimGlobalObject.opt_set_wrap(value);
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(RsvimOpt.prototype, "lineBreak", {
        get: function () {
            return __InternalRsvimGlobalObject.opt_get_line_break();
        },
        set: function (value) {
            if (typeof value !== "boolean") {
                throw new Error("\"Rsvim.opt.lineBreak\" value must be boolean type, but found ".concat(infra.stringify(value)));
            }
            __InternalRsvimGlobalObject.opt_set_line_break(value);
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(RsvimOpt.prototype, "breakAt", {
        get: function () {
            return __InternalRsvimGlobalObject.opt_get_break_at();
        },
        set: function (value) {
            if (typeof value !== "string") {
                throw new Error("\"Rsvim.opt.breakAt\" value must be string type, but found ".concat(infra.stringify(value)));
            }
            __InternalRsvimGlobalObject.opt_set_break_at(value);
        },
        enumerable: false,
        configurable: true
    });
    return RsvimOpt;
}());
export { RsvimOpt };
(function (globalThis) {
    globalThis.Rsvim = new Rsvim();
})(globalThis);
