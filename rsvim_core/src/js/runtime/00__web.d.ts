export declare class TextEncoder {
    #private;
    constructor();
    encode(input: string): Uint8Array;
}
export interface GlobalThis {
    clearInterval(id: number): void;
    clearTimeout(id: number): void;
    queueMicrotask(callback: () => void): void;
    reportError(error: any): void;
    setInterval(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    setTimeout(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    TextEncoder: typeof TextEncoder;
}
