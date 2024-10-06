//! Js runtimes for `Rsvim` namespace.

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
  // Utils {
  /**
   * @param {*} value
   * @returns {string}
   */
  function stringify(value: any): string {
    if (typeof value === "string") {
      return `string["${value}"]`;
    }

    if (typeof value === "number") {
      if (Number.isInteger(value)) {
        return `int[${value}]`;
      }

      return `float[${value}]`;
    }

    if (typeof value === "boolean") {
      return `boolean[${value ? "true" : "false"}]`;
    }

    if (typeof value === "function") {
      return `function[${value.toString()}]`;
    }

    if (typeof value === "object") {
      if (Array.isArray(value)) {
        return `array[length: ${value.length}]`;
      }

      if (value instanceof Map) {
        return `Map[size: ${value.size}]`;
      }

      if (value instanceof WeakMap) {
        return `WeakMap[]`;
      }

      if (value instanceof Set) {
        return `Set[size: ${value.size}]`;
      }

      if (value instanceof WeakSet) {
        return `WeakSet[]`;
      }

      if (value instanceof String) {
        return `String["${value}"]`;
      }

      if (value instanceof Number) {
        let source = value.valueOf();

        if (Number.isInteger(source)) {
          return `Number:int[${source}]`;
        }

        return `Number:float[${source}]`;
      }

      if (value instanceof Boolean) {
        return `Boolean[${value.valueOf() ? "true" : "false"}]`;
      }

      if (value instanceof Date) {
        return `Date["${value.toUTCString()}"]`;
      }

      if (value instanceof RegExp) {
        return `RegExp[${value.toString()}]`;
      }

      return `object[${JSON.stringify(value)}]`;
    }

    if (typeof value === "undefined") {
      return "undefined";
    }

    throw `Unhandled type ${typeof value}`;
  }
  // Utils }

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
        `"Rsvim.opt.lineWrap" value must be boolean type, but found ${stringify(value)}`,
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
