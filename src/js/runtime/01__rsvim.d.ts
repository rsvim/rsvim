//! Js runtime type declarations for `Rsvim` namespace.

export {};

// The `__InternalRsvimGlobalObject.opt` exposed in rust side.
type __InternalRsvimOptionGlobalObject = {
  line_wrap: () => boolean;
  set_line_wrap: (value: boolean) => void;
};

// The `__InternalRsvimGlobalObject` exposed in rust side.
type __InternalRsvimGlobalObject = {
  opt: __InternalRsvimOptionGlobalObject;
};

// The `Rsvim.opt` namespace.
type RsvimOption = {
  // Get `line_wrap` option.
  lineWrap: () => boolean;
  // Set `line_wrap` option.
  setLineWrap: (value: boolean) => void;
};

// The `Rsvim` namespace.
type Rsvim = {
  opt: RsvimOption;
};

// Declare the `Rsvim` global object in `globalThis`.
declare global {
  const Rsvim: Rsvim;
  const __InternalRsvimGlobalObject: __InternalRsvimGlobalObject;
}
