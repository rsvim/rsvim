export declare class Rsvim {
    readonly buf: RsvimBuf;
    readonly cmd: RsvimCmd;
    readonly fs: RsvimFs;
    readonly opt: RsvimOpt;
    readonly rt: RsvimRt;
}
export declare class RsvimBuf {
    current(): number | undefined;
    list(): number[];
    writeSync(bufId: number): number;
}
export declare class RsvimCmd {
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
export declare class RsvimFs {
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
        #private;
        constructor(handle: any);
        close(): void;
        [Symbol.dispose](): void;
        get isDisposed(): boolean;
        read(buf: Uint8Array): Promise<number | null>;
        readSync(buf: Uint8Array): number | null;
        write(buf: Uint8Array): Promise<number | null>;
        writeSync(buf: Uint8Array): number | null;
    }
}
type FileEncodingOption = "utf-8";
type FileFormatOption = "dos" | "unix" | "mac";
export declare class RsvimOpt {
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
