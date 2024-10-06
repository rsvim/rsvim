"use strict";
(function (globalThis) {
    function stringify(value) {
        if (typeof value === "string") {
            return "string[\"".concat(value, "\"]");
        }
        if (typeof value === "number") {
            if (Number.isInteger(value)) {
                return "int[".concat(value, "]");
            }
            return "float[".concat(value, "]");
        }
        if (typeof value === "boolean") {
            return "boolean[".concat(value ? "true" : "false", "]");
        }
        if (typeof value === "function") {
            return "function[".concat(value.toString(), "]");
        }
        if (typeof value === "object") {
            if (Array.isArray(value)) {
                return "array[length: ".concat(value.length, "]");
            }
            if (value instanceof Map) {
                return "Map[size: ".concat(value.size, "]");
            }
            if (value instanceof WeakMap) {
                return "WeakMap[]";
            }
            if (value instanceof Set) {
                return "Set[size: ".concat(value.size, "]");
            }
            if (value instanceof WeakSet) {
                return "WeakSet[]";
            }
            if (value instanceof String) {
                return "String[\"".concat(value, "\"]");
            }
            if (value instanceof Number) {
                var source = value.valueOf();
                if (Number.isInteger(source)) {
                    return "Number:int[".concat(source, "]");
                }
                return "Number:float[".concat(source, "]");
            }
            if (value instanceof Boolean) {
                return "Boolean[".concat(value.valueOf() ? "true" : "false", "]");
            }
            if (value instanceof Date) {
                return "Date[\"".concat(value.toUTCString(), "\"]");
            }
            if (value instanceof RegExp) {
                return "RegExp[".concat(value.toString(), "]");
            }
            return "object[".concat(JSON.stringify(value), "]");
        }
        if (typeof value === "undefined") {
            return "undefined";
        }
        throw new Error("Unhandled type ".concat(typeof value));
    }
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
            throw new Error("\"setTimeout\" callback must be function type, but found ".concat(stringify(callback)));
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
