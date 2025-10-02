declare function checkNotNull(arg: any, msg: string): void;
declare function checkIsNumber(arg: any, msg: string): void;
declare function checkIsInteger(arg: any, msg: string): void;
declare function checkIsBoolean(arg: any, msg: string): void;
declare function checkIsFunction(arg: any, msg: string): void;
declare function checkIsOptions(arg: any, options: any[], msg: string): void;
declare function boundByIntegers(arg: any, bound: [number, number]): any;
declare interface GlobalThis {
    clearInterval(id: number): void;
    clearTimeout(id: number): void;
    queueMicrotask(callback: () => void): void;
    reportError(error: any): void;
    setInterval(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    setTimeout(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
}
