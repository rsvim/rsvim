//! Js runtime type declarations for `Rsvim` namespace.

interface __InternalRsvimGlobalObjectType {
  global_set_timeout(callback: (...args: any[]) => void, delay: number): number;
  global_clear_timeout(id: number): void;
}

declare global {
  const __InternalRsvimGlobalObject: __InternalRsvimGlobalObjectType;
}

export {};
