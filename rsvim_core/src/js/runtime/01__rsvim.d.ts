export declare class Rsvim {
    readonly buf: RsvimBuf;
    readonly cmd: RsvimCmd;
    readonly opt: RsvimOpt;
    readonly rt: RsvimRt;
}
export declare namespace Rsvim {
    namespace opt {
        enum FileEncodingOption {
            UTF_8 = "utf-8"
        }
        enum FileFormatOption {
            DOS = "dos",
            UNIX = "unix",
            MAC = "mac"
        }
    }
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
    get fileEncoding(): Rsvim.opt.FileEncodingOption;
    set fileEncoding(value: Rsvim.opt.FileEncodingOption);
    get fileFormat(): Rsvim.opt.FileFormatOption;
    set fileFormat(value: Rsvim.opt.FileFormatOption);
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
