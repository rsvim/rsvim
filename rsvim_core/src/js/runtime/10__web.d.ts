export interface GlobalThis {
    setTimeout(callback: (...args: any[]) => void, delay: number, ...args: any[]): number;
    clearTimeout(id: number): void;
}
