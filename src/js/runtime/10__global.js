import infra from "rsvim:ext/infra";
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
            throw new Error("\"setTimeout\" callback must be function type, but found ".concat(infra.stringify(callback)));
        }
        var id = nextTimerId++;
        var timer = __InternalRsvimGlobalObject.set_timeout(function () {
            callback.apply(void 0, args);
            activeTimers.delete(id);
        }, delay);
        activeTimers.set(id, timer);
        return id;
    }
    function clearTimeout(id) {
        if (!Number.isInteger(id)) {
            throw new Error("\"clearTimeout\" id must be integer type, but found ".concat(infra.stringify(id)));
        }
        if (activeTimers.has(id)) {
            __InternalRsvimGlobalObject.clear_timeout(activeTimers.get(id));
            activeTimers.delete(id);
        }
    }
    globalThis.setTimeout = setTimeout;
    globalThis.clearTimeout = clearTimeout;
})(globalThis);
