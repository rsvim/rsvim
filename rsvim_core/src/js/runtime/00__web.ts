/**
 * The [WinterTC](https://wintertc.org/) compatible web platform APIs.
 *
 * @see [Minimum Common Web Platform API](https://common-min-api.proposal.wintertc.org/)
 * @see [MDN | Web APIs](https://developer.mozilla.org/docs/Web/API)
 *
 * @packageDocumentation
 */

/** @hidden */
function checkNotNull(arg: any, msg: string) {
  if (arg === undefined || arg === null) {
    throw new TypeError(`${msg} cannot be undefined or null`);
  }
}

/** @hidden */
function checkIsNumber(arg: any, msg: string) {
  if (typeof arg !== "number") {
    throw new TypeError(`${msg} must be a number, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsInteger(arg: any, msg: string) {
  checkIsNumber(arg, msg);
  if (!Number.isInteger(arg)) {
    throw new TypeError(`${msg} must be an integer, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsBoolean(arg: any, msg: string) {
  if (typeof arg !== "boolean") {
    throw new TypeError(`${msg} must be a boolean, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsFunction(arg: any, msg: string) {
  if (typeof arg !== "function") {
    throw new TypeError(`${msg} must be a function, but found ${typeof arg}`);
  }
}

/** @hidden */
function checkIsOptions(arg: any, options: any[], msg: string) {
  if (!options.includes(arg)) {
    throw new RangeError(`${msg} is invalid option: ${arg}`);
  }
}

/** @hidden */
function boundByIntegers(arg: any, bound: [number, number]) {
  if (arg < bound[0]) {
    return bound[0];
  }
  if (arg > bound[1]) {
    return bound[1];
  }
  return arg;
}

/**
 * The [globalThis](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/globalThis) global object.
 */
declare interface GlobalThis {
  /**
   * Cancel a repeated timer previously established by calling {@link setInterval}.
   *
   * @param {number} id - The ID (integer) which identifies the schedule.
   * @throws Throws {@link !TypeError} if ID is not an integer.
   */
  clearInterval(id: number): void;

  /**
   * Cancel a timeout previously established by calling {@link setTimeout}.
   *
   * @param {number} id - The ID (integer) which identifies the timer.
   * @throws Throws {@link !TypeError} if ID is not an integer.
   */
  clearTimeout(id: number): void;

  /**
   * A microtask is a short function which is executed after the function or module which created it exits and
   * only if the JavaScript execution stack is empty, but before returning control to the event loop being used
   * to drive the script's execution environment.
   *
   * @param {function} callback - A function to be executed.
   * @throws Throws {@link !TypeError} if callback is not a function.
   */
  queueMicrotask(callback: () => void): void;

  /**
   * Dispatch an uncaught exception. Similar to synchronous version of `setTimeout(() => {throw error;}, 0);`.
   *
   * @param {any} error - Anything to be thrown.
   */
  reportError(error: any): void;

  /**
   * Set a repeated timer that calls a function, with a fixed time delay between each call.
   *
   * @param {function} callback - A function to be executed every `delay` milliseconds.
   * @param {number} delay - The milliseconds that the timer should delay in between execution of the function. This parameter can be omitted, by default is 1.
   * @param {...any} [args] - Additional arguments which are passed through to the function.
   * @returns {number} The ID (integer) which identifies the timer created.
   * @throws Throws {@link !TypeError} if callback is not a function, or delay is neither a number or undefined.
   */
  setInterval(
    callback: (...args: any[]) => void,
    delay?: number,
    ...args: any[]
  ): number;

  /**
   * Set a timer which executes a function or specified piece of code once the timer expires.
   *
   * @param {function} callback - A function to be executed after the timer expires.
   * @param {number} delay - The milliseconds that the timer should wait before the function is executed. This parameter can be omitted, by default is 1.
   * @param {...any} [args] - Additional arguments which are passed through to the function.
   * @returns {number} The ID (integer) which identifies the timer created.
   * @throws Throws {@link !TypeError} if callback is not a function, or delay is neither a number or undefined.
   */
  setTimeout(
    callback: (...args: any[]) => void,
    delay?: number,
    ...args: any[]
  ): number;
}

((globalThis: GlobalThis) => {
  // Timer API {

  const TIMEOUT_MAX = Math.pow(2, 31) - 1;
  // In javascript side, `nextTimerId` and `activeTimers` maps to the internal
  // timeout ID returned from rust `global_create_timer` api. This is mostly for
  // being compatible with web api standard, as the standard requires the returned
  // value need to be within the range of 1 to 2,147,483,647.
  // See: <https://developer.mozilla.org/en-US/docs/Web/API/Window/setTimeout>.
  let nextTimerId = 1;
  const activeTimers = new Map();

  function clearInterval(id: number): void {
    // Check parameter's type.
    checkIsInteger(id, `"clearInterval" ID`);

    if (activeTimers.has(id)) {
      // @ts-ignore Ignore __InternalRsvimGlobalObject warning
      __InternalRsvimGlobalObject.global_clear_timer(activeTimers.get(id));
      activeTimers.delete(id);
    }
  }

  function setInterval(
    callback: (...args: any[]) => void,
    delay?: number,
    ...args: any[]
  ): number {
    if (delay === undefined || delay === null) {
      delay = 1;
    }
    checkIsNumber(delay, `"setInterval" delay`);

    // Coalesce to number or NaN.
    delay *= 1;

    // Check delay's boundaries.
    delay = boundByIntegers(delay, [1, TIMEOUT_MAX]);

    // Check if callback is a valid function.
    checkIsFunction(callback, `"setInterval" callback`);

    // Pin down the correct ID value.
    const id = nextTimerId++;

    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    const timer = __InternalRsvimGlobalObject.global_create_timer(
      () => {
        callback(...args);
      },
      delay,
      true,
    );

    // Update `activeTimers` map.
    activeTimers.set(id, timer);

    return id;
  }

  function clearTimeout(id: number): void {
    // Check parameter's type.
    checkIsInteger(id, `"clearTimeout" ID`);

    if (activeTimers.has(id)) {
      // @ts-ignore Ignore __InternalRsvimGlobalObject warning
      __InternalRsvimGlobalObject.global_clear_timer(activeTimers.get(id));
      activeTimers.delete(id);
    }
  }

  function setTimeout(
    callback: (...args: any[]) => void,
    delay?: number,
    ...args: any[]
  ): number {
    if (delay === undefined || delay === null) {
      delay = 1;
    }
    checkIsNumber(delay, `"setTimeout" delay`);

    // Coalesce to number or NaN.
    delay *= 1;

    // Check delay's boundaries.
    delay = boundByIntegers(delay, [1, TIMEOUT_MAX]);

    // Check if callback is a valid function.
    checkIsFunction(callback, `"setTimeout" callback`);

    // Pin down the correct ID value.
    const id = nextTimerId++;

    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    const timer = __InternalRsvimGlobalObject.global_create_timer(
      () => {
        callback(...args);
        activeTimers.delete(id);
      },
      delay,
      false,
    );

    // Update `activeTimers` map.
    activeTimers.set(id, timer);

    return id;
  }

  // Timer API }

  // Note: We wrap `queueMicrotask` and manually emit the exception because
  // v8 doesn't provide any mechanism to handle callback exceptions during
  // the microtask_checkpoint phase.
  function queueMicrotask(callback: () => void): void {
    // Check if the callback argument is a valid type.
    checkIsFunction(callback, `"queueMicrotask" callback`);

    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    __InternalRsvimGlobalObject.global_queue_microtask(() => {
      try {
        callback();
      } catch (err) {
        reportError(err);
      }
    });
  }

  function reportError(error: any): void {
    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    __InternalRsvimGlobalObject.global_report_error(error);
  }

  globalThis.clearTimeout = clearTimeout;
  globalThis.setTimeout = setTimeout;
  globalThis.clearInterval = clearInterval;
  globalThis.setInterval = setInterval;
  globalThis.queueMicrotask = queueMicrotask;
  globalThis.reportError = reportError;
})(globalThis as unknown as GlobalThis);
