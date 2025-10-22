type TextEncoderEncodeIntoResult = {
    read: number;
    written: number;
};
export declare class TextEncoder {
    constructor();
    get encoding(): string;
    encode(input: string): Uint8Array;
    encodeInto(src: string, dest: Uint8Array): TextEncoderEncodeIntoResult;
}
type TextDecoderOptions = {
    fatal?: boolean;
    ignoreBOM?: boolean;
};
export declare class TextDecoder {
    #private;
    constructor(encoding?: string, options?: TextDecoderOptions);
    encode(input: string): Uint8Array;
    encodeInto(src: string, dest: Uint8Array): TextEncoderEncodeIntoResult;
}
export interface GlobalThis {
    clearInterval(id: number): void;
    clearTimeout(id: number): void;
    queueMicrotask(callback: () => void): void;
    reportError(error: any): void;
    setInterval(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    setTimeout(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
}
export {};
