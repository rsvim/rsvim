export declare class Rsvim {
    readonly cmd: RsvimCmd;
    readonly opt: RsvimOpt;
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
