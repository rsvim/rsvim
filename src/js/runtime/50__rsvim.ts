//! Js runtimes for `Rsvim` namespace.

// @ts-ignore Ignore internal import warning
import infra from "rsvim:ext/infra";

// `Rsvim.opt`
interface RsvimOptionType {
  lineWrap(): boolean;
  setLineWrap(value: boolean): void;
}

// `Rsvim`
interface GlobalThisType {
  opt: RsvimOptionType;
}

(function (globalThis: { Rsvim: GlobalThisType }) {
  // `Rsvim.opt` {

  // Get `line_wrap` option.
  function optLineWrap(): boolean {
    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    return __InternalRsvimGlobalObject.opt_line_wrap();
  }

  // `Rsvim.opt.setLineWrap`
  //
  // Set `line_wrap` option.
  function optSetLineWrap(value: boolean): void {
    if (typeof value !== "boolean") {
      throw new Error(
        `"Rsvim.opt.lineWrap" value must be boolean type, but found ${infra.stringify(value)}`,
      );
    }
    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    __InternalRsvimGlobalObject.opt_set_line_wrap(value);
  }

  // `Rsvim.opt` }

  globalThis.Rsvim = {
    opt: {
      lineWrap: optLineWrap,
      setLineWrap: optSetLineWrap,
    } as RsvimOptionType,
  } as GlobalThisType;
})(globalThis as unknown as { Rsvim: GlobalThisType });
