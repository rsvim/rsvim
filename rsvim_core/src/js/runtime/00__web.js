function isNull(arg) {
    return arg === undefined || arg === null;
}
function isString(arg) {
    return typeof arg === "string";
}
function checkNotNull(arg, msg) {
    if (isNull(arg)) {
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
    if (!isString(arg)) {
        throw new TypeError(`${msg} must be a string, but found ${typeof arg}`);
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
function checkIsUint8Array(arg, msg) {
    if (!(arg instanceof Uint8Array)) {
        throw new TypeError(`${msg} must be a Uint8Array, buf found ${typeof arg}`);
    }
}
function isTypedArray(arg) {
    return (arg instanceof Int8Array ||
        arg instanceof Uint8Array ||
        arg instanceof Uint8ClampedArray ||
        arg instanceof Int16Array ||
        arg instanceof Uint16Array ||
        arg instanceof Int32Array ||
        arg instanceof Uint32Array ||
        arg instanceof Float16Array ||
        arg instanceof Float32Array ||
        arg instanceof Float64Array ||
        arg instanceof BigInt64Array ||
        arg instanceof BigUint64Array);
}
function isArrayBuffer(arg) {
    return arg instanceof ArrayBuffer;
}
function isDataView(arg) {
    return arg instanceof DataView;
}
function checkIsArrayBufferFamily(arg, msg) {
    if (!(isArrayBuffer(arg) || isDataView(arg) || isTypedArray(arg))) {
        throw new TypeError(`${msg} must be either ArrayBuffer/DataView/TypedArray, buf found ${typeof arg}`);
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
function setDefaultFields(arg, defaults) {
    for (const [key, val] of Object.entries(defaults)) {
        if (!Object.hasOwn(arg, key)) {
            Object.defineProperty(arg, key, val);
        }
    }
}
export class TextEncoder {
    constructor() { }
    encode(input) {
        checkIsString(input, `"TextEncoder.encode" input`);
        return __InternalRsvimGlobalObject.global_encoding_encode(input);
    }
    encodeInto(src, dest) {
        checkIsString(src, `"TextEncoder.encodeInto" src`);
        checkIsUint8Array(dest, `"TextEncoder.encodeInto" dest`);
        return __InternalRsvimGlobalObject.global_encoding_encode_into(src, dest.buffer);
    }
    get encoding() {
        return "utf-8";
    }
}
export class TextDecoder {
    #handle;
    #encoding;
    #fatal;
    #ignoreBOM;
    constructor(encoding, options) {
        encoding = encoding ?? "utf-8";
        checkIsString(encoding, `"TextDecoder.constructor" encoding`);
        const encodingIsValid = __InternalRsvimGlobalObject.global_encoding_check_encoding_label(encoding);
        if (!encodingIsValid) {
            throw new RangeError(`"TextDecoder.constructor" encoding is unknown: ${encoding}`);
        }
        options = options ?? { fatal: false, ignoreBOM: false };
        checkIsObject(options, `"TextDecoder.constructor" options`);
        setDefaultFields(options, { fatal: false, ignoreBOM: false });
        checkIsBoolean(options.fatal, `"TextDecoder.constructor" fatal option`);
        checkIsBoolean(options.ignoreBOM, `"TextDecoder.constructor" ignoreBOM option`);
        this.#encoding = encoding;
        this.#fatal = options.fatal;
        this.#ignoreBOM = options.ignoreBOM;
        this.#handle = null;
    }
    decode(input, options = { stream: false }) {
        input = input ?? new Uint8Array();
        checkIsArrayBufferFamily(input, `"TextDecoder.decode" input`);
        let buffer = input;
        if (isTypedArray(input)) {
            buffer = input.buffer;
        }
        else if (isDataView(input)) {
            buffer = input.buffer;
        }
        options = options ?? { stream: false };
        checkIsObject(options, `"TextDecoder.decode" options`);
        setDefaultFields(options, { stream: false });
        checkIsBoolean(options.stream, `"TextDecoder.decode" stream option`);
        const stream = options.stream;
        try {
            if (!stream && this.#handle === null) {
                return __InternalRsvimGlobalObject.global_encoding_decode_single(buffer, this.#encoding, this.#fatal, this.#ignoreBOM);
            }
            if (this.#handle === null) {
                this.#handle =
                    __InternalRsvimGlobalObject.global_encoding_create_stream_decoder(this.#encoding, this.#ignoreBOM);
            }
            return __InternalRsvimGlobalObject.global_encoding_decode_stream(buffer, this.#handle, this.#fatal, stream);
        }
        finally {
            if (!stream && this.#handle !== null) {
                this.#handle = null;
            }
        }
    }
    get encoding() {
        return this.#encoding;
    }
    get fatal() {
        return this.#fatal;
    }
    get ignoreBOM() {
        return this.#ignoreBOM;
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
        delay = delay ?? 1;
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
        delay = delay ?? 1;
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
