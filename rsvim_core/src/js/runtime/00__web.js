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
            throw new Error("\"setTimeout\" callback parameter must be a function, but found ".concat(callback, " (").concat(typeof callback, ")"));
        }
        var id = nextTimerId++;
        var timer = __InternalRsvimGlobalObject.global_set_timeout(function () {
            callback.apply(void 0, args);
            activeTimers.delete(id);
        }, delay);
        activeTimers.set(id, timer);
        return id;
    }
    function clearTimeout(id) {
        if (!Number.isInteger(id)) {
            throw new Error("\"clearTimeout\" id parameter must be an integer value, but found ".concat(id, " (").concat(typeof id, ")"));
        }
        if (activeTimers.has(id)) {
            __InternalRsvimGlobalObject.global_clear_timeout(activeTimers.get(id));
            activeTimers.delete(id);
        }
    }
    globalThis.setTimeout = setTimeout;
    globalThis.clearTimeout = clearTimeout;
})(globalThis);
export {};
