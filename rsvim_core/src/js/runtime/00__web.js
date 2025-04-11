((globalThis) => {
    const TIMEOUT_MAX = Math.pow(2, 31) - 1;
    let nextTimerId = 1;
    const activeTimers = new Map();
    function setTimeout(callback, delay, ...args) {
        delay *= 1;
        if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
            delay = 1;
        }
        if (typeof callback !== "function") {
            throw new Error(`"setTimeout" callback parameter must be a function, but found ${callback} (${typeof callback})`);
        }
        const id = nextTimerId++;
        const timer = __InternalRsvimGlobalObject.global_set_timeout(() => {
            callback(...args);
            activeTimers.delete(id);
        }, delay);
        activeTimers.set(id, timer);
        return id;
    }
    function clearTimeout(id) {
        if (!Number.isInteger(id)) {
            throw new Error(`"clearTimeout" id parameter must be an integer value, but found ${id} (${typeof id})`);
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
