// RSVIM js runtime.
((globalThis) => {
  const core = Deno.core;
  core.initializeAsyncOps();

  function argsToMessage(...args: any[]) {
    return args.map((arg) => JSON.stringify(arg)).join(" ");
  }

  globalThis.console = {
    log: (...args) => {
      core.print(`[out]: ${argsToMessage(...args)}\n`, false);
    },
    error: (...args) => {
      core.print(`[err]: ${argsToMessage(...args)}\n`, true);
    },
  };

  // globalThis.vim = {
  //   fs: {
  //     readFile: (path: string) => {
  //       return core.ops.op_read_file(path);
  //     },
  //     writeFile: (path: string, contents: string) => {
  //       return core.ops.op_write_file(path, contents);
  //     },
  //     removeFile: (path: string) => {
  //       return core.ops.op_remove_file(path);
  //     },
  //   },
  // };
})(globalThis);
