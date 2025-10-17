export declare class Rsvim {
    readonly buf: RsvimBufImpl;
    readonly cmd: RsvimCmdImpl;
    readonly fs: RsvimFsImpl;
    readonly opt: RsvimOptImpl;
    readonly rt: RsvimRtImpl;
}
export interface RsvimBuf {
    current(): number | undefined;
    list(): number[];
    writeSync(bufId: number): number;
}
declare class RsvimBufImpl implements RsvimBuf {
    current(): number | undefined;
    list(): number[];
    writeSync(bufId: number): number;
}
export interface RsvimCmd {
    create(name: string, callback: RsvimCmd.CommandCallback, attributes?: RsvimCmd.CommandAttributes, options?: RsvimCmd.CommandOptions): RsvimCmd.CommandDefinition | undefined;
    echo(message: any): void;
    list(): string[];
    get(name: string): RsvimCmd.CommandDefinition | undefined;
    remove(name: string): RsvimCmd.CommandDefinition | undefined;
}
declare class RsvimCmdImpl implements RsvimCmd {
    create(name: string, callback: RsvimCmd.CommandCallback, attributes?: RsvimCmd.CommandAttributes, options?: RsvimCmd.CommandOptions): RsvimCmd.CommandDefinition | undefined;
    echo(message: any): void;
    list(): string[];
    get(name: string): RsvimCmd.CommandDefinition | undefined;
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
export interface RsvimFs {
    open(path: string, options?: RsvimFs.OpenOptions): Promise<RsvimFs.File>;
    openSync(path: string, options?: RsvimFs.OpenOptions): RsvimFs.File;
}
declare class RsvimFsImpl implements RsvimFs {
    open(path: string, options?: RsvimFs.OpenOptions): Promise<RsvimFs.File>;
    openSync(path: string, options?: RsvimFs.OpenOptions): RsvimFs.File;
}
export declare namespace RsvimFs {
    type OpenOptions = {
        append?: boolean;
        create?: boolean;
        createNew?: boolean;
        read?: boolean;
        truncate?: boolean;
        write?: boolean;
    };
    class File {
        __file_handle: any;
        constructor(__file_handle: any);
        close(): void;
        isClosed(): boolean;
    }
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
declare class RsvimOptImpl implements RsvimOpt {
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
declare class RsvimRtImpl implements RsvimRt {
    exit(exitCode?: number): void;
}
export {};
