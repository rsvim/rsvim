function checkNotNull(arg, msg) {
    if (arg === undefined || arg === null) {
        throw new TypeError(`${msg} cannot be undefined or null`);
    }
}
function checkIsNumber(arg, msg) {
    if (typeof arg !== "number") {
        throw new TypeError(`${msg} must be a number, but found ${typeof arg}`);
    }
}
function checkIsInteger(arg, msg) {
    checkIsNumber(arg, msg);
    if (!Number.isInteger(arg)) {
        throw new TypeError(`${msg} must be an integer, but found ${typeof arg}`);
    }
}
function checkIsBoolean(arg, msg) {
    if (typeof arg !== "boolean") {
        throw new TypeError(`${msg} must be a boolean, but found ${typeof arg}`);
    }
}
function checkIsString(arg, msg) {
    if (typeof arg !== "string") {
        throw new TypeError(`${msg} must be a string, but found ${typeof arg}`);
    }
}
function checkIsUint8Array(arg, msg) {
    if (arg instanceof Uint8Array) {
        throw new TypeError(`${msg} must be a Uint8Array, but found ${typeof arg}`);
    }
}
function checkIsFunction(arg, msg) {
    if (typeof arg !== "function") {
        throw new TypeError(`${msg} must be a function, but found ${typeof arg}`);
    }
}
function checkIsObject(arg, msg) {
    if (typeof arg !== "object") {
        throw new TypeError(`${msg} must be an object, but found ${typeof arg}`);
    }
}
function checkIsOptions(arg, options, msg) {
    if (!options.includes(arg)) {
        throw new RangeError(`${msg} is invalid option: ${arg}`);
    }
}
function boundByIntegers(arg, bound) {
    if (arg < bound[0]) {
        return bound[0];
    }
    if (arg > bound[1]) {
        return bound[1];
    }
    return arg;
}
export class TextEncoder {
    constructor() { }
    get encoding() {
        return "utf-8";
    }
    encode(input) {
        checkIsString(input, `"TextEncoder.encode" input`);
        return __InternalRsvimGlobalObject.global_encoding_encode(input);
    }
    encodeInto(src, dest) {
        checkIsString(src, `"TextEncoder.encodeInto" src`);
        checkIsUint8Array(src, `"TextEncoder.encodeInto" dest`);
        return __InternalRsvimGlobalObject.global_encoding_encode_into(input, dest);
    }
}
export class TextDecoder {
    #handle;
    constructor(encoding, options) {
        checkIsString(encoding, `"TextDecoder.constructor" encoding`);
        if (options === undefined || options === null) {
            options = { fatal: false, ignoreBOM: false };
        }
        checkIsObject(options, `"TextDecoder.constructor" options`);
        if (!Object.hasOwn(options, "fatal")) {
            options.fatal = false;
        }
        if (!Object.hasOwn(options, "ignoreBOM")) {
            options.ignoreBOM = false;
        }
        const handle = __InternalRsvimGlobalObject.global_encoding_create_decoder(encoding, options);
        this.#handle = handle;
    }
    encode(input) {
        checkIsString(input, `"TextEncoder.encode" input`);
        return __InternalRsvimGlobalObject.global_encoding_encode(input);
    }
    encodeInto(src, dest) {
        checkIsString(src, `"TextEncoder.encodeInto" src`);
        checkIsUint8Array(src, `"TextEncoder.encodeInto" dest`);
        return __InternalRsvimGlobalObject.global_encoding_encode_into(input, dest);
    }
}
((globalThis) => {
    const TIMEOUT_MAX = Math.pow(2, 31) - 1;
    let nextTimerId = 1;
    const activeTimers = new Map();
    function clearInterval(id) {
        checkIsInteger(id, `"clearInterval" ID`);
        if (activeTimers.has(id)) {
            __InternalRsvimGlobalObject.global_clear_timer(activeTimers.get(id));
            activeTimers.delete(id);
        }
    }
    function setInterval(callback, delay, ...args) {
        if (delay === undefined || delay === null) {
            delay = 1;
        }
        checkIsNumber(delay, `"setInterval" delay`);
        delay *= 1;
        delay = boundByIntegers(delay, [1, TIMEOUT_MAX]);
        checkIsFunction(callback, `"setInterval" callback`);
        const id = nextTimerId++;
        const timer = __InternalRsvimGlobalObject.global_create_timer(() => {
            callback(...args);
        }, delay, true);
        activeTimers.set(id, timer);
        return id;
    }
    function clearTimeout(id) {
        checkIsInteger(id, `"clearTimeout" ID`);
        if (activeTimers.has(id)) {
            __InternalRsvimGlobalObject.global_clear_timer(activeTimers.get(id));
            activeTimers.delete(id);
        }
    }
    function setTimeout(callback, delay, ...args) {
        if (delay === undefined || delay === null) {
            delay = 1;
        }
        checkIsNumber(delay, `"setTimeout" delay`);
        delay *= 1;
        delay = boundByIntegers(delay, [1, TIMEOUT_MAX]);
        checkIsFunction(callback, `"setTimeout" callback`);
        const id = nextTimerId++;
        const timer = __InternalRsvimGlobalObject.global_create_timer(() => {
            callback(...args);
            activeTimers.delete(id);
        }, delay, false);
        activeTimers.set(id, timer);
        return id;
    }
    function queueMicrotask(callback) {
        checkIsFunction(callback, `"queueMicrotask" callback`);
        __InternalRsvimGlobalObject.global_queue_microtask(() => {
            try {
                callback();
            }
            catch (err) {
                reportError(err);
            }
        });
    }
    function reportError(error) {
        __InternalRsvimGlobalObject.global_report_error(error);
    }
    globalThis.clearTimeout = clearTimeout;
    globalThis.setTimeout = setTimeout;
    globalThis.clearInterval = clearInterval;
    globalThis.setInterval = setInterval;
    globalThis.queueMicrotask = queueMicrotask;
    globalThis.reportError = reportError;
    globalThis.TextEncoder = TextEncoder;
    globalThis.TextDecoder = TextDecoder;
})(globalThis);
