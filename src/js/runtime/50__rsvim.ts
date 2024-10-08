//! Js runtimes for `Rsvim` namespace.

// @ts-ignore Ignore internal import warning
import infra from "rsvim:ext/infra";

// `Rsvim.opt`
class RsvimOption {
  // Get `line_wrap` option.
  get lineWrap(): boolean {
    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    return __InternalRsvimGlobalObject.opt_line_wrap();
  }

  // Set `line_wrap` option.
  set lineWrap(value: boolean) {
    if (typeof value !== "boolean") {
      throw new Error(
        `"Rsvim.opt.lineWrap" value must be boolean type, but found ${infra.stringify(value)}`,
      );
    }
    // @ts-ignore Ignore __InternalRsvimGlobalObject warning
    __InternalRsvimGlobalObject.opt_set_line_wrap(value);
  }
}

// `Rsvim`
interface GlobalThisType {
  opt: RsvimOption;
}

(function (globalThis: { Rsvim: GlobalThisType }) {
  globalThis.Rsvim = {
    opt: new RsvimOption(),
  } as GlobalThisType;
})(globalThis as unknown as { Rsvim: GlobalThisType });
