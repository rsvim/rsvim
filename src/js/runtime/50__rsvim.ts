/**
 * The global namespace for `Rsvim` specific, non-standard WinterCG APIs.
 *
 * @packageDocumentation
 *
 * @categoryDescription Editor-related APIs
 * These APIs are specific for Rsvim editors such as buffers, windows, statusline, etc.
 *
 * @categoryDescription General purpose APIs
 * These APIs are general purpose for common JavaScript and TypeScript runtime, similar to node.js and deno.
 */

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
 * The `Rsvim.opt` global editor options, also see {@link Rsvim}.
 *
 * @category Editor-related APIs.
 */
export interface RsvimOpt {
  /**
   * Get the _line-wrap_ option.
   *
   * @returns {boolean} The _line-wrap_ option.
   * @defaultValue `false`.
   */
  lineWrap(): boolean;

  /**
   * Set the _line-wrap_ option.
   *
   * @param {boolean} value - The _line-wrap_ option.
   * @throws {@link !Error} if value is not a boolean value.
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
