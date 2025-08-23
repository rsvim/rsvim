export declare class Rsvim {
    readonly buf: RsvimBuf;
    readonly cmd: RsvimCmd;
    readonly opt: RsvimOpt;
}
export declare class RsvimBuf {
    current(): number | undefined;
    list(): number[];
    writeSync(bufId: number): void;
}
export declare class RsvimCmd {
    echo(message: string): void;
}
export declare class RsvimOpt {
    get lineBreak(): boolean;
    set lineBreak(value: boolean);
    get wrap(): boolean;
    set wrap(value: boolean);
}
