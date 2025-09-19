export interface Rsvim {
    readonly buf: RsvimBuf;
    readonly cmd: RsvimCmd;
    readonly opt: RsvimOpt;
    readonly rt: RsvimRt;
}
export declare class RsvimImpl implements Rsvim {
    buf: RsvimBufImpl;
    cmd: RsvimCmdImpl;
    opt: RsvimOptImpl;
    rt: RsvimRt;
}
export interface RsvimBuf {
    current(): number | null;
    list(): number[];
    writeSync(bufId: number): number;
}
export declare class RsvimBufImpl implements RsvimBuf {
    current(): number | null;
    list(): number[];
    writeSync(bufId: number): number;
}
export interface RsvimCmd {
    echo(message: string): void;
}
export declare class RsvimCmdImpl implements RsvimCmd {
    echo(message: string): void;
}
type FileEncodingOption = "utf-8";
type FileFormatOption = "dos" | "unix" | "mac";
export interface RsvimOpt {
    get expandTab(): boolean;
    set expandTab(value: boolean);
    get fileEncoding(): FileEncodingOption;
    set fileEncoding(value: FileEncodingOption);
    get fileFormat(): FileFormatOption;
    set fileFormat(value: FileFormatOption);
    get lineBreak(): boolean;
    set lineBreak(value: boolean);
    get shiftWidth(): number;
    set shiftWidth(value: number);
    get tabStop(): number;
    set tabStop(value: number);
    get wrap(): boolean;
    set wrap(value: boolean);
}
export declare class RsvimOptImpl implements RsvimOpt {
    get expandTab(): boolean;
    set expandTab(value: boolean);
    get fileEncoding(): FileEncodingOption;
    set fileEncoding(value: FileEncodingOption);
    get fileFormat(): FileFormatOption;
    set fileFormat(value: FileFormatOption);
    get lineBreak(): boolean;
    set lineBreak(value: boolean);
    get shiftWidth(): number;
    set shiftWidth(value: number);
    get tabStop(): number;
    set tabStop(value: number);
    get wrap(): boolean;
    set wrap(value: boolean);
}
export declare class RsvimRt {
    exit(exitCode?: number): void;
}
export {};
