//! Js runtime type declarations for `Rsvim` namespace.

interface __InternalRsvimGlobalObjectType {
  opt_line_wrap(): boolean;
  opt_set_line_wrap(value: boolean): void;
}

declare global {
  const __InternalRsvimGlobalObject: __InternalRsvimGlobalObjectType;
}

export {};
