"use strict";
//! Js runtimes for `Rsvim` namespace.
(function (globalThis) {
    // `Rsvim`
    globalThis.Rsvim = {
        // `Rsvim.opt`
        opt: {
            lineWrap: function () {
                // @ts-ignore
                return __InternalRsvimGlobalObject.opt_line_wrap();
            },
            setLineWrap: function (value) {
                if (typeof value !== "boolean") {
                    throw new Error("Value (".concat(value, ") must be boolean type, but found ").concat(typeof value));
                }
                // @ts-ignore
                __InternalRsvimGlobalObject.opt_set_line_wrap(value);
            },
        },
    };
})(globalThis);
