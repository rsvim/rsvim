//! Js runtime type declarations for `Rsvim` namespace.

// The `Rsvim.opt` namespace.
declare class RsvimOption {
  // Get `line_wrap` option.
  lineWrap(): boolean;
  // Set `line_wrap` option.
  setLineWrap(value: boolean): void;
}

// The `Rsvim` namespace.
declare class Rsvim {
  opt: RsvimOption;
}

// Declare the `Rsvim` global object in `globalThis`.
declare global {
  const Rsvim: Rsvim;
}

export {};
