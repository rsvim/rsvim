/**
 * The [WinterTC](https://wintertc.org/) compatible web platform APIs.
 *
 * @see [Minimum Common Web Platform API](https://common-min-api.proposal.wintertc.org/)
 * @see [MDN | Web APIs](https://developer.mozilla.org/en-US/docs/Web/API)
 *
 * @packageDocumentation
 */

/**
 * The {@link !globalThis} global object.
 */
export interface GlobalThis {
  /**
   * Cancel a repeated scheduler previously established by calling {@link setInterval}.
   *
   * @param {number} id - The ID (integer) which identifies the schedule.
   * @throws Throws {@link !Error} if ID is not an integer value.
   */
  clearInterval(id: number): void;

  /**
   * Cancel a timeout previously established by calling {@link setTimeout}.
   *
   * @param {number} id - The ID (integer) which identifies the timer.
   * @throws Throws {@link !Error} if ID is not an integer value.
   */
  clearTimeout(id: number): void;

  /**
   * Set a repeated scheduler that calls a function, with a fixed time delay between each call. Also see {@link !setInterval}.
   *
   * @param {Function} callback - A function to be executed every `delay` milliseconds.
   * @param {number} delay - The milliseconds that the scheduler should wait before the function is executed.
   * @param {...any} [args] - Additional arguments which are passed through to the function.
   * @returns {number} The ID (integer) which identifies the timer created.
   * @throws Throws {@link !Error} if callback is not a function value.
   */
  setInterval(
    callback: (...args: any[]) => void,
    delay: number,
    ...args: any[]
  ): number;

  /**
   * Set a timer which executes a function or specified piece of code once the timer expires. Also see {@link !setTimeout}.
   *
   * @param {Function} callback - A function to be executed after the timer expires.
   * @param {number} delay - The milliseconds that the timer should wait before the function is executed.
   * @param {...any} [args] - Additional arguments which are passed through to the function.
   * @returns {number} The ID (integer) which identifies the timer created.
   * @throws Throws {@link !Error} if callback is not a function value.
   */
  setTimeout(
    callback: (...args: any[]) => void,
    delay: number,
    ...args: any[]
  ): number;
}

((globalThis: GlobalThis) => {
  // Timer API {

  const TIMEOUT_MAX = Math.pow(2, 31) - 1;
  // In javascript side, `nextTimerId` and `activeTimers` maps to the internal
  // timeout ID returned from rust `global_set_timeout` api. This is mostly for
  // being compatible with web api standard, as the standard requires the returned
  // value need to be within the range of 1 to 2,147,483,647.
  // See: <https://developer.mozilla.org/en-US/docs/Web/API/Window/setTimeout>.
  let nextTimerId = 1;
  const activeTimers = new Map();

  function clearTimeout(id: number): void {
    // Check parameter's type.
    if (!Number.isInteger(id)) {
      throw new Error(
        `"clearTimeout" id parameter must be an integer value, but found ${id} (${typeof id})`,
      );
    }

    if (activeTimers.has(id)) {
      // @ts-ignore Ignore __InternalRsvimGlobalObject warning
      __InternalRsvimGlobalObject.global_clear_timeout(activeTimers.get(id));
      activeTimers.delete(id);
    }
  }

  function setTimeout(
    callback: (...args: any[]) => void,
    delay: number,
    ...args: any[]
  ): number {
    // Coalesce to number or NaN.
    delay *= 1;

    // Check delay's boundaries.
    if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
      delay = 1;
    }

    // Check if callback is a valid function.
    if (typeof callback !== "function") {
      throw new Error(
        `"setTimeout" callback parameter must be a function, but found ${callback} (${typeof callback})`,
      );
    }

    // Pin down the correct ID value.
    const id = nextTimerId++;

    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    const timer = __InternalRsvimGlobalObject.global_set_timeout(() => {
      callback(...args);
      activeTimers.delete(id);
    }, delay);

    // Update `activeTimers` map.
    activeTimers.set(id, timer);

    return id;
  }

  // Timer API }

  // const { $$queueMicrotask, reportError } = globalThis;
  //
  // // Note: We wrap `queueMicrotask` and manually emit the exception because
  // // v8 doesn't provide any mechanism to handle callback exceptions during
  // // the microtask_checkpoint phase.
  // function queueMicrotask(callback) {
  //   // Check if the callback argument is a valid type.
  //   if (typeof callback !== "function") {
  //     throw new TypeError(`The "callback" argument must be a function.`);
  //   }
  //
  //   $$queueMicrotask(() => {
  //     try {
  //       callback();
  //     } catch (err) {
  //       reportError(err);
  //     }
  //   });
  // }

  globalThis.clearTimeout = clearTimeout;
  globalThis.setTimeout = setTimeout;
})(globalThis as unknown as GlobalThis);
