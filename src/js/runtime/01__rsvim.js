"use strict";
(function (globalThis) {
    globalThis.Rsvim = {
        opt: {
            lineWrap: function () {
                return __InternalRsvimGlobalObject.opt_line_wrap();
            },
            setLineWrap: function (value) {
                if (typeof value !== "boolean") {
                    throw new Error("Value (".concat(value, ") must be boolean type, but found ").concat(typeof value));
                }
                __InternalRsvimGlobalObject.opt_set_line_wrap(value);
            },
        },
    };
})(globalThis);
