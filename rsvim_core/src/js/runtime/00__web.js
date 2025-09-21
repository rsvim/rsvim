((globalThis) => {
    const TIMEOUT_MAX = Math.pow(2, 31) - 1;
    let nextTimerId = 1;
    const activeTimers = new Map();
    function clearInterval(id) {
        if (!Number.isInteger(id)) {
            throw new TypeError(`"clearInterval" id must be an integer, but found ${typeof id}`);
        }
        if (activeTimers.has(id)) {
            __InternalRsvimGlobalObject.global_clear_timer(activeTimers.get(id));
            activeTimers.delete(id);
        }
    }
    function clearTimeout(id) {
        if (!Number.isInteger(id)) {
            throw new TypeError(`"clearTimeout" id must be an integer, but found ${typeof id}`);
        }
        if (activeTimers.has(id)) {
            __InternalRsvimGlobalObject.global_clear_timer(activeTimers.get(id));
            activeTimers.delete(id);
        }
    }
    function setTimeout(callback, delay, ...args) {
        if (delay === undefined || delay === null) {
            delay = 1;
        }
        else if (typeof delay !== "number") {
            throw new TypeError(`"setTimeout" delay must be a number, but found ${typeof delay}`);
        }
        delay *= 1;
        if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
            delay = 1;
        }
        if (typeof callback !== "function") {
            throw new Error(`"setTimeout" callback must be a function, but found ${typeof callback}`);
        }
        const id = nextTimerId++;
        const timer = __InternalRsvimGlobalObject.global_create_timer(() => {
            callback(...args);
            activeTimers.delete(id);
        }, delay, false);
        activeTimers.set(id, timer);
        return id;
    }
    function setInterval(callback, delay, ...args) {
        if (delay === undefined || delay === null) {
            delay = 1;
        }
        else if (typeof delay !== "number") {
            throw new TypeError(`"setInterval" delay must be a number, but found ${typeof delay}`);
        }
        delay *= 1;
        if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
            delay = 1;
        }
        if (typeof callback !== "function") {
            throw new Error(`"setTimeout" callback must be a function, but found ${typeof callback}`);
        }
        const id = nextTimerId++;
        const timer = __InternalRsvimGlobalObject.global_create_timer(() => {
            callback(...args);
        }, delay, true);
        activeTimers.set(id, timer);
        return id;
    }
    globalThis.clearTimeout = clearTimeout;
    globalThis.setTimeout = setTimeout;
})(globalThis);
export {};
