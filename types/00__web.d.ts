export declare namespace TextEncoder {
    type EncodeIntoResult = {
        read: number;
        written: number;
    };
}
export declare class TextEncoder {
    constructor();
    encode(input: string): Uint8Array;
    encodeInto(src: string, dest: Uint8Array): TextEncoder.EncodeIntoResult;
    get encoding(): string;
}
export declare namespace TextDecoder {
    type Options = {
        fatal?: boolean;
        ignoreBOM?: boolean;
    };
    type DecodeOptions = {
        stream?: boolean;
    };
}
export declare class TextDecoder {
    #private;
    constructor(encoding?: string, options?: TextDecoder.Options);
    decode(input?: ArrayBuffer | GlobalThis.TypedArray | DataView, options?: TextDecoder.DecodeOptions): string;
    get encoding(): string;
    get fatal(): boolean;
    get ignoreBOM(): boolean;
}
export declare namespace GlobalThis {
    type TypedArray = Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array | BigInt64Array | BigUint64Array;
}
export interface GlobalThis {
    clearInterval(id: number): void;
    clearTimeout(id: number): void;
    queueMicrotask(callback: () => void): void;
    reportError(error: any): void;
    setInterval(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    setTimeout(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    TextEncoder: TextEncoder;
    TextDecoder: TextDecoder;
}
