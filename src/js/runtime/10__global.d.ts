//! Js runtime type declarations for `Rsvim` namespace.

interface __InternalRsvimGlobalObjectType {
  set_timeout(callback: (...args: any[]) => void, delay: number): number;
}

declare global {
  const __InternalRsvimGlobalObject: __InternalRsvimGlobalObjectType;
}

export {};
