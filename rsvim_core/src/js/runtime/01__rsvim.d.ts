export declare class Rsvim {
    readonly buf: RsvimBuf;
    readonly cmd: RsvimCmd;
    readonly opt: RsvimOpt;
    readonly rt: RsvimRt;
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
    get fileEncoding(): "utf-8";
    set fileEncoding(value: "utf-8");
    get fileFormat(): "dos" | "unix" | "mac";
    set fileFormat(value: "dos" | "unix" | "mac");
    get lineBreak(): boolean;
    set lineBreak(value: boolean);
    get tabStop(): number;
    set tabStop(value: number);
    get wrap(): boolean;
    set wrap(value: boolean);
}
export declare class RsvimRt {
    exit(exitCode?: number): void;
}
