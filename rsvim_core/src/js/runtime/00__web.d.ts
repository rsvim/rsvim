export interface GlobalThis {
    clearTimeout(id: number): void;
    setTimeout(callback: (...args: any[]) => void, delay: number, ...args: any[]): number;
}
