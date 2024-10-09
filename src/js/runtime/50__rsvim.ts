// Js runtimes for `Rsvim` namespace.

// @ts-ignore Ignore internal import warning
import infra from "rsvim:ext/infra";

/**
 * The type definition for global object `Rsvim`.
 *
 * It contains multiple sub fields:
 *
 * - `Rsvim.opt`: Global editor options, see {@link RsvimOpt}.
 */
export interface Rsvim {
  opt: RsvimOpt;
}

/**
 * The type definition for global object `Rsvim.opt`.
 *
 * See {@link Rsvim}.
 */
export interface RsvimOpt {
  /**
   * Get editor line-wrap option.
   *
   * @returns The line-wrap option value.
   *
   * @defaultValue `false`.
   */
  lineWrap(): boolean;

  /**
   * Set editor line-wrap option.
   *
   * @param value - line-wrap value.
   *
   * @throws {@link Error} if {@link value} is not boolean.
   */
  setLineWrap(value: boolean): void;
}

(function (globalThis: { Rsvim: Rsvim }) {
  globalThis.Rsvim = {
    opt: {
      lineWrap: function (): boolean {
        // @ts-ignore Ignore __InternalRsvimGlobalObject warning
        return __InternalRsvimGlobalObject.opt_line_wrap();
      },
      setLineWrap: function (value: boolean): void {
        if (typeof value !== "boolean") {
          throw new Error(
            `"Rsvim.opt.lineWrap" value must be boolean type, but found ${infra.stringify(value)}`,
          );
        }
        // @ts-ignore Ignore __InternalRsvimGlobalObject warning
        __InternalRsvimGlobalObject.opt_set_line_wrap(value);
      },
    } as RsvimOpt,
  } as Rsvim;
})(globalThis as unknown as { Rsvim: Rsvim });
