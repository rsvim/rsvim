export declare class Rsvim {
    readonly buf: RsvimBuf;
    readonly cmd: RsvimCmd;
    readonly opt: RsvimOpt;
}
export declare class RsvimBuf {
    current(): number | null;
    list(): number[];
    writeSync(bufId: number): number;
}
export declare class RsvimCmd {
    echo(message: string): number;
}
export declare class RsvimOpt {
    get lineBreak(): boolean;
    set lineBreak(value: boolean);
    get wrap(): boolean;
    set wrap(value: boolean);
}
