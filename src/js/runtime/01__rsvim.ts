//! Js runtimes for `Rsvim` namespace.

((globalThis) => {
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

  // `Rsvim`
  globalThis.Rsvim = {
    // `Rsvim.opt`
    opt = {
      lineWrap: (): boolean => __InternalRsvimGlobalObject.line_wrap(),
      setLineWrap: (value: boolean): void => {
        if (typeof value !== "boolean") {
          throw new Error(
            `Value (${value}) must be boolean type, but found ${typeof value}`,
          );
        }
        __InternalRsvimGlobalObject.set_line_wrap(value);
      },
    },
  };
})(globalThis);
