/**
 * The [WinterCG](https://common-min-api.proposal.wintercg.org/) compatible web platform APIs.
 *
 * @packageDocumentation
 */

// @ts-ignore Ignore internal import warning
import infra from "rsvim:ext/infra";

/**
 * The {@link !globalThis} global namespace.
 */
export interface GlobalThis {
  /**
   * Sets a timer which executes a function or specified piece of code once the timer expires. Also see {@link !setTimeout}.
   *
   * @param {Function} callback - A function to be executed after the timer expires.
   * @param {number} delay - The milliseconds that the timer should wait before the function is executed.
   * @param {...any} [args] - Additional arguments which are passed through to the function.
   * @returns {number} The ID (integer) which identifies the timer created.
   * @throws {@link !Error} if callback is not a function value.
   */
  setTimeout(
    callback: (...args: any[]) => void,
    delay: number,
    ...args: any[]
  ): number;

  /**
   * Cancels a timeout previously established by calling {@link setTimeout}.
   *
   * @param {number} id - The ID (integer) which identifies the timer.
   * @throws {@link !Error} if ID is not an integer value.
   */
  clearTimeout(id: number): void;
}

((globalThis: GlobalThis) => {
  // Timer API {

  const TIMEOUT_MAX = Math.pow(2, 31) - 1;
  let nextTimerId = 1;
  const activeTimers = new Map();

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
        `"setTimeout" callback must be function type, but found ${infra.stringify(callback)}`,
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

  function clearTimeout(id: number): void {
    // Check parameter's type.
    if (!Number.isInteger(id)) {
      throw new Error(
        `"clearTimeout" id must be integer type, but found ${infra.stringify(id)}`,
      );
    }

    if (activeTimers.has(id)) {
      // @ts-ignore Ignore __InternalRsvimGlobalObject warning
      __InternalRsvimGlobalObject.global_clear_timeout(activeTimers.get(id));
      activeTimers.delete(id);
    }
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

  globalThis.setTimeout = setTimeout;
  globalThis.clearTimeout = clearTimeout;
})(globalThis as unknown as GlobalThis);
