//! Js runtime type declarations for `Rsvim` namespace.

// `Rsvim.opt`
declare type RsvimOption = {
  // Get `line_wrap` option.
  lineWrap: () => boolean;
  // Set `line_wrap` option.
  setLineWrap: (value: boolean) => void;
};

// Declare `Rsvim`.
declare global {
  // `Rsvim`.
  interface Rsvim {
    // `Rsvim.opt`.
    opt: RsvimOption;
  }
}

export {};
