// RSVIM runtime operations.
(function (globalThis) {
    var core = Deno.core;
    function argsToMessage() {
        var args = [];
        for (var _i = 0; _i < arguments.length; _i++) {
            args[_i] = arguments[_i];
        }
        return args.map(function (arg) { return JSON.stringify(arg); }).join(" ");
    }
    globalThis.console = {
        log: function () {
            var args = [];
            for (var _i = 0; _i < arguments.length; _i++) {
                args[_i] = arguments[_i];
            }
            core.print("[out]: ".concat(argsToMessage.apply(void 0, args), "\n"), false);
        },
        error: function () {
            var args = [];
            for (var _i = 0; _i < arguments.length; _i++) {
                args[_i] = arguments[_i];
            }
            core.print("[err]: ".concat(argsToMessage.apply(void 0, args), "\n"), true);
        },
    };
})(globalThis);
