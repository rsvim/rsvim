//! Js runtimes for `Rsvim` namespace.

interface RsvimOptionType {
  lineWrap(): boolean;
  setLineWrap(value: boolean): void;
}

interface RsvimType {
  opt: RsvimOptionType;
}

(function (globalThis: { Rsvim: RsvimType }) {
  // `Rsvim`
  globalThis.Rsvim = {
    // `Rsvim.opt`
    opt: {
      lineWrap: function () {
        return __InternalRsvimGlobalObject.opt_line_wrap();
      },
      setLineWrap: function (value) {
        if (typeof value !== "boolean") {
          throw new Error(
            `Value (${value}) must be boolean type, but found ${typeof value}`,
          );
        }
        __InternalRsvimGlobalObject.opt_set_line_wrap(value);
      },
    },
  } as RsvimType;
})(globalThis as unknown as { Rsvim: RsvimType });
