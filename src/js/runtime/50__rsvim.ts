// Js runtimes for `Rsvim` namespace.

// @ts-ignore Ignore internal import warning
import infra from "rsvim:ext/infra";

/**
 * The `Rsvim` global object, it contains multiple sub fields:
 *
 * - `Rsvim.opt`: Global editor options, also see {@link RsvimOpt}.
 */
export interface Rsvim {
  opt: RsvimOpt;
}

/**
 * The `Rsvim.opt` global object, also see {@link Rsvim}.
 */
export interface RsvimOpt {
  /**
   * Get **line-wrap** option.
   *
   * @returns The **line-wrap** option.
   *
   * @defaultValue `false`.
   */
  lineWrap(): boolean;

  /**
   * Set **line-wrap** option.
   *
   * @param value - The **line-wrap** option.
   *
   * @throws Error if value is not a boolean value.
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
