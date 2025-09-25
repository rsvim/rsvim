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
    create(name: string, callback: RsvimCmd.CommandCallback, attr?: RsvimCmd.CommandAttributes, opts?: RsvimCmd.CreateCommandOptions): undefined | RsvimCmd.CommandCallback;
    echo(message: any): void;
}
export declare namespace RsvimCmd {
    type CommandAttributes = {
        bang?: boolean;
        nargs?: "0" | "1" | "?" | "+" | "?";
        bufId?: number;
    };
    type CreateCommandOptions = {
        force?: boolean;
    };
    type CommandCallback = (ctx: any) => void;
}
export interface RsvimOpt {
    get expandTab(): boolean;
    set expandTab(value: boolean);
    get fileEncoding(): RsvimOpt.FileEncodingOption;
    set fileEncoding(value: RsvimOpt.FileEncodingOption);
    get fileFormat(): RsvimOpt.FileFormatOption;
    set fileFormat(value: RsvimOpt.FileFormatOption);
    get lineBreak(): boolean;
    set lineBreak(value: boolean);
    get shiftWidth(): number;
    set shiftWidth(value: number);
    get tabStop(): number;
    set tabStop(value: number);
    get wrap(): boolean;
    set wrap(value: boolean);
}
export declare namespace RsvimOpt {
    type FileEncodingOption = "utf-8";
    type FileFormatOption = "dos" | "unix" | "mac";
}
export interface RsvimRt {
    exit(exitCode?: number): void;
}
