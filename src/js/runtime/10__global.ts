//! Js runtimes for global object, i.e. `globalThis`.

// @ts-ignore Ignore internal import warning
import infra from "rsvim:ext/infra";

// `globalThis`
interface GlobalThisType {
  setTimeout(
    callback: (...args: any[]) => void,
    delay: number,
    ...args: any[]
  ): void;
}

((globalThis) => {
  // Timer API {

  const TIMEOUT_MAX = Math.pow(2, 31) - 1;
  let nextTimerId = 1;
  const activeTimers = new Map();

  /**
   * Sets a timer which executes a function or specified piece of code once the
   * timer expires.
   *
   * @param {Function} callback - A function to be executed after the timer expires.
   * @param {Number} delay - The milliseconds that the timer should wait before the function is executed.
   * @param {...any} [args] - Additional arguments which are passed through to the function.
   * @returns {Number} The ID which identifies the timer created.
   */
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
    const timer = __InternalRsvimGlobalObject.set_timeout(() => {
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

  globalThis.setTimeout = setTimeout;
})(globalThis as unknown as GlobalThisType);
