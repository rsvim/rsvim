export declare namespace GlobalThis {
    type TypedArray = Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array | BigInt64Array | BigUint64Array;
}
type TextEncoderEncodeIntoResult = {
    read: number;
    written: number;
};
export declare class TextEncoder {
    constructor();
    encode(input: string): Uint8Array;
    encodeInto(src: string, dest: Uint8Array): TextEncoderEncodeIntoResult;
    get encoding(): string;
}
type TextDecoderOptions = {
    fatal?: boolean;
    ignoreBOM?: boolean;
};
type TextDecoderDecodeOptions = {
    stream?: boolean;
};
export declare class TextDecoder {
    #private;
    constructor(encoding?: string, options?: TextDecoderOptions);
    decode(input: ArrayBuffer | GlobalThis.TypedArray | DataView, options?: TextDecoderDecodeOptions): string;
    get encoding(): string;
    get fatal(): boolean;
    get ignoreBOM(): string;
}
export interface GlobalThis {
    clearInterval(id: number): void;
    clearTimeout(id: number): void;
    queueMicrotask(callback: () => void): void;
    reportError(error: any): void;
    setInterval(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    setTimeout(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    TextEncoder: typeof TextEncoder;
    TextDecoder: typeof TextDecoder;
}
export {};
