export interface GlobalThis {
    clearInterval(id: number): void;
    clearTimeout(id: number): void;
    setInterval(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
    setTimeout(callback: (...args: any[]) => void, delay?: number, ...args: any[]): number;
}
