import infra from "rsvim:ext/infra";
(function (globalThis) {
    globalThis.Rsvim = {
        opt: {
            lineWrap: function () {
                return __InternalRsvimGlobalObject.opt_line_wrap();
            },
            setLineWrap: function (value) {
                if (typeof value !== "boolean") {
                    throw new Error("\"Rsvim.opt.lineWrap\" value must be boolean type, but found ".concat(infra.stringify(value)));
                }
                __InternalRsvimGlobalObject.opt_set_line_wrap(value);
            },
        },
    };
})(globalThis);
