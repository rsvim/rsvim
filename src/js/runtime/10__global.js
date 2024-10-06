"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var infra_1 = require("rsvim:ext/infra");
(function (globalThis) {
    var TIMEOUT_MAX = Math.pow(2, 31) - 1;
    var nextTimerId = 1;
    var activeTimers = new Map();
    function setTimeout(callback, delay) {
        var args = [];
        for (var _i = 2; _i < arguments.length; _i++) {
            args[_i - 2] = arguments[_i];
        }
        delay *= 1;
        if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
            delay = 1;
        }
        if (typeof callback !== "function") {
            throw new Error("\"setTimeout\" callback must be function type, but found ".concat(infra_1.default.stringify(callback)));
        }
        var id = nextTimerId++;
        var timer = __InternalRsvimGlobalObject.set_timeout(function () {
            callback.apply(void 0, args);
            activeTimers.delete(id);
        }, delay);
        activeTimers.set(id, timer);
        return id;
    }
    globalThis.setTimeout = setTimeout;
})(globalThis);
