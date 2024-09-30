//! Js runtimes for `Rsvim` namespace.

// `Rsvim.opt`
interface RsvimOptionType {
  lineWrap(): boolean;
  setLineWrap(value: boolean): void;
}

// `Rsvim`
interface RsvimType {
  opt: RsvimOptionType;
}

// `Rsvim.opt.lineWrap`
//
// Get `line_wrap` option.
function optLineWrap(): boolean {
  // @ts-ignore
  return __InternalRsvimGlobalObject.opt_line_wrap();
}

// `Rsvim.opt.setLineWrap`
//
// Set `line_wrap` option.
function optSetLineWrap(value: boolean): void {
  if (typeof value !== "boolean") {
    throw new Error(
      `Value (${value}) must be boolean type, but found ${typeof value}`,
    );
  }
  // @ts-ignore
  __InternalRsvimGlobalObject.opt_set_line_wrap(value);
}

(function (globalThis: { Rsvim: RsvimType }) {
  globalThis.Rsvim = {
    opt: {
      lineWrap: optLineWrap,
      setLineWrap: optSetLineWrap,
    } as RsvimOptionType,
  } as RsvimType;
})(globalThis as unknown as { Rsvim: RsvimType });
