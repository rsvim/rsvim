export interface Rsvim {
    readonly buf: RsvimBuf;
    readonly cmd: RsvimCmd;
    readonly opt: RsvimOpt;
    readonly rt: RsvimRt;
}
export interface RsvimBuf {
    current(): number | null;
    list(): number[];
    writeSync(bufId: number): number;
}
export interface RsvimCmd {
    echo(message: any): void;
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
export interface RsvimRt {
    exit(exitCode?: number): void;
}
export {};
