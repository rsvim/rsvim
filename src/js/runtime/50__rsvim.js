"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var infra_1 = require("rsvim:ext/infra");
(function (globalThis) {
    function optLineWrap() {
        return __InternalRsvimGlobalObject.opt_line_wrap();
    }
    function optSetLineWrap(value) {
        if (typeof value !== "boolean") {
            throw new Error("\"Rsvim.opt.lineWrap\" value must be boolean type, but found ".concat(infra_1.default.stringify(value)));
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
