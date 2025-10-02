export declare interface Rsvim {
    readonly buf: RsvimBuf;
    readonly cmd: RsvimCmd;
    readonly opt: RsvimOpt;
    readonly rt: RsvimRt;
}
export interface RsvimBuf {
    current(): number | undefined;
    list(): number[];
    writeSync(bufId: number): number;
}
export interface RsvimCmd {
    create(name: string, callback: RsvimCmd.CommandCallback, attributes?: RsvimCmd.CommandAttributes, options?: RsvimCmd.CommandOptions): RsvimCmd.CommandDefinition | undefined;
    echo(message: any): void;
    list(): RsvimCmd.CommandDefinition[];
    remove(name: string): RsvimCmd.CommandDefinition | undefined;
}
export declare namespace RsvimCmd {
    type CommandAttributes = {
        bang?: boolean;
        nargs?: "0" | "1" | "*" | "+" | "?";
    };
    type CommandOptions = {
        force?: boolean;
        alias?: string;
    };
    type CommandCallback = (ctx: any) => void;
    type CommandDefinition = {
        name: string;
        callback: CommandCallback;
        attributes: CommandAttributes;
        options: CommandOptions;
    };
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
