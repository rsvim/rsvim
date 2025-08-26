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
type FileEncodingOption = "utf-8";
export declare class RsvimOpt {
    get fileEncoding(): FileEncodingOption;
    set fileEncoding(value: FileEncodingOption);
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
export {};
