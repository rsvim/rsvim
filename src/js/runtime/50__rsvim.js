import infra from "rsvim:ext/infra";
(function (globalThis) {
    globalThis.Rsvim = {
        opt: {
            getWrap: function () {
                return __InternalRsvimGlobalObject.opt_get_wrap();
            },
            setWrap: function (value) {
                if (typeof value !== "boolean") {
                    throw new Error("\"Rsvim.opt.setWrap\" value must be boolean type, but found ".concat(infra.stringify(value)));
                }
                __InternalRsvimGlobalObject.opt_set_wrap(value);
            },
            getLineBreak: function () {
                return __InternalRsvimGlobalObject.opt_get_line_break();
            },
            setLineBreak: function (value) {
                if (typeof value !== "boolean") {
                    throw new Error("\"Rsvim.opt.setLineBreak\" value must be boolean type, but found ".concat(infra.stringify(value)));
                }
                __InternalRsvimGlobalObject.opt_set_line_break(value);
            },
        },
    };
})(globalThis);
