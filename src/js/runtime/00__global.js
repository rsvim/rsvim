"use strict";
//! Js runtimes for global object, i.e. `globalThis`.
(function (globalThis) {
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
})(globalThis);
