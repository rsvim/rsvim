import infra from "rsvim:ext/infra";
var RsvimOption = (function () {
    function RsvimOption() {
    }
    Object.defineProperty(RsvimOption.prototype, "lineWrap", {
        get: function () {
            return __InternalRsvimGlobalObject.opt_line_wrap();
        },
        set: function (value) {
            if (typeof value !== "boolean") {
                throw new Error("\"Rsvim.opt.lineWrap\" value must be boolean type, but found ".concat(infra.stringify(value)));
            }
            __InternalRsvimGlobalObject.opt_set_line_wrap(value);
        },
        enumerable: false,
        configurable: true
    });
    return RsvimOption;
}());
(function (globalThis) {
    globalThis.Rsvim = {
        opt: new RsvimOption(),
    };
})(globalThis);
