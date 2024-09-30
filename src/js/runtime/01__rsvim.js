"use strict";
(function (globalThis) {
    function optLineWrap() {
        return __InternalRsvimGlobalObject.opt_line_wrap();
    }
    function optSetLineWrap(value) {
        if (typeof value !== "boolean") {
            throw new Error("Value (".concat(value, ") must be boolean type, but found ").concat(typeof value));
        }
        __InternalRsvimGlobalObject.opt_set_line_wrap(value);
    }
    globalThis.Rsvim = {
        opt: {
            lineWrap: optLineWrap,
            setLineWrap: optSetLineWrap,
        },
    };
})(globalThis);
