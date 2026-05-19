## [0.1.3-alpha.1](https://github.com/rsvim/rsvim/compare/v0.1.2..0.1.3-alpha.1) - 2026-05-19

[79d7cd5c](https://github.com/rsvim/rsvim/commit/79d7cd5cc7a93d9633a6df603379e3b5714b79e5)...[1f2ca2c9](https://github.com/rsvim/rsvim/commit/1f2ca2c997dbbaef05ddbf8e7019c95a534d975e)

### <!-- 0 -->Features

- *(cli)* Provide v8 js engine version info in cli version (#210) ([9cb3cbc1](https://github.com/rsvim/rsvim/commit/9cb3cbc15e6f9f50452fb9a7eacc1305d71f8e60)) by @linrongbin16 ([#210](https://github.com/rsvim/rsvim/pull/210))

- *(ui)* Add viewport for better mapping buffer to window (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(buf)* Add "tabstop" local options for buffer (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(grapheme)* Add ascii control codes (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(unicode)* Add char display width calculation (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(ui)* Drop "breakat" option (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(ui)* Render "wrap" and "linebreak" in viewport (#218) ([c0dca1a2](https://github.com/rsvim/rsvim/commit/c0dca1a2e8939ed4ae14a7a845d19e731a2145e5)) by @linrongbin16 ([#218](https://github.com/rsvim/rsvim/pull/218))

- *(buf)* Impl "new file" edit operation for buffers (#229) ([cad5833f](https://github.com/rsvim/rsvim/commit/cad5833f7c7f35c16d84747d98769542f1a21a1d)) by @linrongbin16 ([#229](https://github.com/rsvim/rsvim/pull/229))

- *(ui)* Maps every char index to its display boundary columns in viewport (#244) ([4ce805f1](https://github.com/rsvim/rsvim/commit/4ce805f113f896b90c4be1f694d73db94a388c97)) by @linrongbin16 ([#244](https://github.com/rsvim/rsvim/pull/244))

- *(cursor)* Impl cursor movement in normal editing mode - part 1 (#277) ([94af6e19](https://github.com/rsvim/rsvim/commit/94af6e19d138845c41170f39b9e5a6b93696f079)) by @linrongbin16 ([#277](https://github.com/rsvim/rsvim/pull/277))

- *(cursor)* Add widget node motion api (#296) ([8311468f](https://github.com/rsvim/rsvim/commit/8311468f923aca3e3db7036e9aa61d022d8c95a2)) by @linrongbin16 ([#296](https://github.com/rsvim/rsvim/pull/296))

- *(cursor)* Impl cursor motion in normal mode - part 3 (#297) ([91810bd7](https://github.com/rsvim/rsvim/commit/91810bd7d63fcea9ab3ba6eaf52e6261be6eb8af)) by @linrongbin16 ([#297](https://github.com/rsvim/rsvim/pull/297))

- *(win)* Impl line-break/word-wrap window rendering (again) (#312) ([89f28077](https://github.com/rsvim/rsvim/commit/89f28077228192bb1b1dc11ef1e8092bff8a96d4)) by @linrongbin16 ([#312](https://github.com/rsvim/rsvim/pull/312))

- *(cursor)* Scroll buffer when cursor reaches window top/bottom (nowrap) (#322) ([31711f21](https://github.com/rsvim/rsvim/commit/31711f215fd19c60c16d98588a8a65e41a476b2e)) by @linrongbin16 ([#322](https://github.com/rsvim/rsvim/pull/322))

- *(cursor)* Impl cursor horizontal scroll when reaches window border (#342) ([7f6bdd4a](https://github.com/rsvim/rsvim/commit/7f6bdd4a7961b2182ae08c2ffbe2773f2670d9ee)) by @linrongbin16 ([#342](https://github.com/rsvim/rsvim/pull/342))

- *(cursor)* Add more cursor motions and enable 2D position in a single move (#348) ([f3586c9a](https://github.com/rsvim/rsvim/commit/f3586c9a9153aa95bda24ecba5bf668351dd447b)) by @linrongbin16 ([#348](https://github.com/rsvim/rsvim/pull/348))

- *(cursor)* Impl cursor motion with buffer scrolling when reaches window border (#354) ([2045529d](https://github.com/rsvim/rsvim/commit/2045529d5104de14861d97a082a447c88887a6f5)) by @linrongbin16 ([#354](https://github.com/rsvim/rsvim/pull/354))

- *(viewport)* Add search new viewport anchor(start line/column) downward api (#365) ([40d8729b](https://github.com/rsvim/rsvim/commit/40d8729bca08e0f01cf9b6e1e57deed1b045a633)) by @linrongbin16 ([#365](https://github.com/rsvim/rsvim/pull/365))

- *(cursor)* Add search viewport anchor upward api (#369) ([f231a899](https://github.com/rsvim/rsvim/commit/f231a8996cf5356e9550dbfb432b9cefa384d504)) by @linrongbin16 ([#369](https://github.com/rsvim/rsvim/pull/369))

- *(viewport)* Add search viewport anchor leftward/rightward (#373) ([6d803f74](https://github.com/rsvim/rsvim/commit/6d803f74483c9440bcff3addfee9efbd1e4f13e2)) by @linrongbin16 ([#373](https://github.com/rsvim/rsvim/pull/373))

- *(cursor)* Provide more operation primitives for normal mode (#382) ([103288db](https://github.com/rsvim/rsvim/commit/103288dbda07480c40492b17096cf81cabf5a01e)) by @linrongbin16 ([#382](https://github.com/rsvim/rsvim/pull/382))

- *(insert)* Allow i/esc keycode to switch between insert/normal mode (#391) ([c0fd13fd](https://github.com/rsvim/rsvim/commit/c0fd13fd4b08a97ccf9649e0fa8b0108b282503f)) by @linrongbin16 ([#391](https://github.com/rsvim/rsvim/pull/391))

- *(insert)* Initialize cursor movement in insert mode (#393) ([4dbc4b49](https://github.com/rsvim/rsvim/commit/4dbc4b49b779ae729f23a26ff5d9ef33fbc4da13)) by @linrongbin16 ([#393](https://github.com/rsvim/rsvim/pull/393))

- *(viewport)* Give 1 extra column for empty eol (end-of-line) (#424) ([9fadf4f1](https://github.com/rsvim/rsvim/commit/9fadf4f11accd5a02a18b8fcc6db1adf9f1f41f0)) by @linrongbin16 ([#424](https://github.com/rsvim/rsvim/pull/424))

- *(cursor)* Include empty eol when calculate cursor positions for insert mode (#435) ([b5861e84](https://github.com/rsvim/rsvim/commit/b5861e844aff233096ae2083e644bfb3c2aa0789)) by @linrongbin16 ([#435](https://github.com/rsvim/rsvim/pull/435))

- *(cursor)* Impl cursor moves to empty eol when 'wrap=true' (#437) ([f94f960b](https://github.com/rsvim/rsvim/commit/f94f960b93014d2bdfa65355a4df255ed5e41aaa)) by @linrongbin16 ([#437](https://github.com/rsvim/rsvim/pull/437))

- *(insert)* Insert char for insert mode (#444) ([035ca45c](https://github.com/rsvim/rsvim/commit/035ca45ce3d9df8e7d06b1b2a6c6ce3c7b8af9f9)) by @linrongbin16 ([#444](https://github.com/rsvim/rsvim/pull/444))

- *(insert)* Insert eol when file has no eol at end (#445) ([f2a45bb8](https://github.com/rsvim/rsvim/commit/f2a45bb807348213dc4e01f2a6edee095b81e806)) by @linrongbin16 ([#445](https://github.com/rsvim/rsvim/pull/445))

- *(insert)* Insert line break with enter (#452) ([2e2adb42](https://github.com/rsvim/rsvim/commit/2e2adb429177c09168ec409ef6d1f6020d164dda)) by @linrongbin16 ([#452](https://github.com/rsvim/rsvim/pull/452))

- *(insert)* Delete text at cursor (#453) ([3545d505](https://github.com/rsvim/rsvim/commit/3545d505ba30b28ae105d1a3301a32055633a89b)) by @linrongbin16 ([#453](https://github.com/rsvim/rsvim/pull/453))

- *(cmdline)* Add 'Cmdline' widget (#469) ([20c67017](https://github.com/rsvim/rsvim/commit/20c67017d614b5f3f87154014389495017efcf44)) by @linrongbin16 ([#469](https://github.com/rsvim/rsvim/pull/469))

- *(cmdline)* Initialize default cmdline widget (#471) ([f4ac9b52](https://github.com/rsvim/rsvim/commit/f4ac9b523c75a4b815d63323714bbbf41ef91158)) by @linrongbin16 ([#471](https://github.com/rsvim/rsvim/pull/471))

- *(cmdline)* Add 'GotoCommandLineMode' and ex/search variants operation (#480) ([7f771d8b](https://github.com/rsvim/rsvim/commit/7f771d8b2a2cce94eed63eb857615b2f66872ee8)) by @linrongbin16 ([#480](https://github.com/rsvim/rsvim/pull/480))

- *(cmdline)* Add more command-line ex mode operations (#481) ([3e5835bd](https://github.com/rsvim/rsvim/commit/3e5835bdbba3c2bfddfa95ddca269cbd82d7f1cb)) by @linrongbin16 ([#481](https://github.com/rsvim/rsvim/pull/481))

- *(cmdline)* Goto command-line ex mode (#482) ([d2eebc29](https://github.com/rsvim/rsvim/commit/d2eebc294d227e77440d488915433a4256d77811)) by @linrongbin16 ([#482](https://github.com/rsvim/rsvim/pull/482))

- *(v8)* Add default v8 flags (#509) ([13a3795a](https://github.com/rsvim/rsvim/commit/13a3795a7914699e637cf9064988958f8ae0e70b)) by @linrongbin16 ([#509](https://github.com/rsvim/rsvim/pull/509))

- *(normal)* Implement append insert ("a" key) and new line insert ("o" key) (#533) ([7b20d308](https://github.com/rsvim/rsvim/commit/7b20d30840901d9b36fc4b150c14c7f908f9fe34)) by @jackcat13 ([#533](https://github.com/rsvim/rsvim/pull/533))

- *(js/evloop/fsm/ui)* Add "echo" command and command_line message widget (#559) ([50e08ca3](https://github.com/rsvim/rsvim/commit/50e08ca3ec1f2d4c9b61f1a2763c2849408df1f7)) by @jackcat13 ([#559](https://github.com/rsvim/rsvim/pull/559))

- *(js)* Print error message when failed to load config (#593) ([c40edf61](https://github.com/rsvim/rsvim/commit/c40edf6114abc70d72446877a7a74a029576a41f)) by @linrongbin16 ([#593](https://github.com/rsvim/rsvim/pull/593))

- *(cmd)* Impl builtin `:js` ex command to run any js script (#598) ([817113a4](https://github.com/rsvim/rsvim/commit/817113a4dde97e2778103d41403e42dd74a5d833)) by @linrongbin16 ([#598](https://github.com/rsvim/rsvim/pull/598))

- *(buf)* Add `Rsvim.buf.writeSync` api (#604) ([34691e92](https://github.com/rsvim/rsvim/commit/34691e92a96215ed57db8c7a31090d4ece77d87f)) by @linrongbin16 ([#604](https://github.com/rsvim/rsvim/pull/604))

- *(exit)* Add "Rsvim.rt.exit()" API to quit editor process (#619) ([1e5eeb0c](https://github.com/rsvim/rsvim/commit/1e5eeb0c7cbb0f471ee1ad422cd1f3dd8612b272)) by @linrongbin16 ([#619](https://github.com/rsvim/rsvim/pull/619))

- *(exit)* No longer press "ESC" to quit editor (#621) ([81c1e92f](https://github.com/rsvim/rsvim/commit/81c1e92fcb1c4a3dc1e9e75484d8a6161b93a973)) by @linrongbin16 ([#621](https://github.com/rsvim/rsvim/pull/621))

- *(opts)* Add more buffer options (#622) ([0d181572](https://github.com/rsvim/rsvim/commit/0d181572156a6f6ce91d0150b0a0ceca19a3a701)) by @linrongbin16 ([#622](https://github.com/rsvim/rsvim/pull/622))

- *(expandtab)* Add "expandTab" and "shiftWidth" to allow inserts spaces instead of tab (#624) ([5ac4f2ae](https://github.com/rsvim/rsvim/commit/5ac4f2aede71e894245848e84d4f2982299a7bc6)) by @linrongbin16 ([#624](https://github.com/rsvim/rsvim/pull/624))

- *(js)* Support dynamic import (#631) ([3cefaae7](https://github.com/rsvim/rsvim/commit/3cefaae7140bfcae23a035927afbb4a5f69bc0e0)) by @linrongbin16 ([#631](https://github.com/rsvim/rsvim/pull/631))

- *(js)* (fully?) support ES module and npm package resolving (#664) ([271d5c55](https://github.com/rsvim/rsvim/commit/271d5c55406da89f9ed287eaa90592de20f75697)) by @linrongbin16 ([#664](https://github.com/rsvim/rsvim/pull/664))

- *(api)* Add "setInterval" api (#675) ([094cb971](https://github.com/rsvim/rsvim/commit/094cb9710bde5d91633b16dc9357beafff0dc719)) by @linrongbin16 ([#675](https://github.com/rsvim/rsvim/pull/675))

- *(api)* Add "queueMicrotask" and "reportError" api (#681) ([439ad8e4](https://github.com/rsvim/rsvim/commit/439ad8e41ef2a076be256ef607039d73c6c8ffda)) by @linrongbin16 ([#681](https://github.com/rsvim/rsvim/pull/681))

- *(api)* Add "Rsvim.cmd.create" apis (#687) ([2107eb8d](https://github.com/rsvim/rsvim/commit/2107eb8da98e16c40951d65e6c248d9b7bbe056d)) by @linrongbin16 ([#687](https://github.com/rsvim/rsvim/pull/687))

- *(cmd)* Impl command alias and v8/rust converter (#698) ([c2a7772a](https://github.com/rsvim/rsvim/commit/c2a7772a9d30ef20960aca3a01a938a5daed0a75)) by @linrongbin16 ([#698](https://github.com/rsvim/rsvim/pull/698))

- *(cmd)* Run user registered ex commands (#728) ([f9ba4da9](https://github.com/rsvim/rsvim/commit/f9ba4da96c928756dea27b2118b55c6ec0839ce0)) by @linrongbin16 ([#728](https://github.com/rsvim/rsvim/pull/728))

- *(fs)* Add `Rsvim.fs` open, openSync, close APIs (#738) ([896a5e9d](https://github.com/rsvim/rsvim/commit/896a5e9d5751678dc2d68e91e28795c99573a89a)) by @linrongbin16 ([#738](https://github.com/rsvim/rsvim/pull/738))

- *(encoding)* Add "TextEncoder" and refactor js converter (#741) ([74078120](https://github.com/rsvim/rsvim/commit/74078120a04efc6e42b66b91f793f408f5245240)) by @linrongbin16 ([#741](https://github.com/rsvim/rsvim/pull/741))

- *(encoding)* Add "TextDecoder" (#749) ([c3b87beb](https://github.com/rsvim/rsvim/commit/c3b87beb6ea6e1689007e11e2bce57645c241c04)) by @linrongbin16 ([#749](https://github.com/rsvim/rsvim/pull/749))

- *(fs)* Add fs read api (#751) ([05cb7aec](https://github.com/rsvim/rsvim/commit/05cb7aec058f1e9e4721b2c7954db6e426173963)) by @linrongbin16 ([#751](https://github.com/rsvim/rsvim/pull/751))

- *(fs)* Add fs write api (#757) ([9d9af6b7](https://github.com/rsvim/rsvim/commit/9d9af6b7a800777acc7e45cc6aba8677e6f63aa7)) by @linrongbin16 ([#757](https://github.com/rsvim/rsvim/pull/757))

- *(types)* Add global declarations for typescript types (#761) ([f014a223](https://github.com/rsvim/rsvim/commit/f014a2234f2ce9bf07957e48c3e7a62afa56339a)) by @linrongbin16 ([#761](https://github.com/rsvim/rsvim/pull/761))

- *(cmd)* Add "CommandContext" (#764) ([4255a14d](https://github.com/rsvim/rsvim/commit/4255a14d5e1f619e1ad6e70c6f27abca50ecf5f7)) by @linrongbin16 ([#764](https://github.com/rsvim/rsvim/pull/764))

- *(buf)* Save editing changes to undo manager (#858) ([feb4b43b](https://github.com/rsvim/rsvim/commit/feb4b43bade3bd269855de871b9f01c82c7cfbee)) by @linrongbin16 ([#858](https://github.com/rsvim/rsvim/pull/858))

- *(undo)* Revert text rope with undo history (#869) ([ac583d9f](https://github.com/rsvim/rsvim/commit/ac583d9fff9fa1df9b39593b446cadea51169d77)) by @linrongbin16 ([#869](https://github.com/rsvim/rsvim/pull/869))

- *(syn)* Parse buffer text with tree-sitter syntax parser (#878) ([a263585d](https://github.com/rsvim/rsvim/commit/a263585d812f4ca0191b816746a9a909b31c7a9d)) by @linrongbin16 ([#878](https://github.com/rsvim/rsvim/pull/878))

- *(syn)* Syntax parser parse text payload in async task (#886) ([3253369c](https://github.com/rsvim/rsvim/commit/3253369c6d783dbd80716fb73aecc4a4b3d81fdc)) by @linrongbin16 ([#886](https://github.com/rsvim/rsvim/pull/886))

- *(syn)* Add editing changes and pend for parsing (#897) ([b272548f](https://github.com/rsvim/rsvim/commit/b272548f90a0b02432a595eb94e982b71ddc8d40)) by @linrongbin16 ([#897](https://github.com/rsvim/rsvim/pull/897))

- *(hl)* Add highlight (#912) ([66366645](https://github.com/rsvim/rsvim/commit/66366645c05965d1cd5a2d30e0ededc3a4617eeb)) by @linrongbin16 ([#912](https://github.com/rsvim/rsvim/pull/912))

- *(hl)* Add default highlight (#920) ([7d905f06](https://github.com/rsvim/rsvim/commit/7d905f0678904553fba3845bb7045ead9ecc2b35)) by @linrongbin16 ([#920](https://github.com/rsvim/rsvim/pull/920))

- *(hl)* Resolve specific highlight per language with fallback (#933) ([1a1961ba](https://github.com/rsvim/rsvim/commit/1a1961ba514e77bc912a2e890cd19bb152702ed4)) by @linrongbin16 ([#933](https://github.com/rsvim/rsvim/pull/933))

- *(syn)* Execute highlight query when parsing syntax (#945) ([dcf25acf](https://github.com/rsvim/rsvim/commit/dcf25acf62cbbcb9f1878fa3420d510333527072)) by @linrongbin16 ([#945](https://github.com/rsvim/rsvim/pull/945))

- *(hl)* Write captured highlight into canvas (#964) ([b835204f](https://github.com/rsvim/rsvim/commit/b835204f5a076785461347b77268397a6996d731)) by @linrongbin16 ([#964](https://github.com/rsvim/rsvim/pull/964))

- *(ui)* Render canvas with color (#974) ([bec16a5c](https://github.com/rsvim/rsvim/commit/bec16a5c77565733eae0936d4a1b9ef721a5e2a1)) by @linrongbin16 ([#974](https://github.com/rsvim/rsvim/pull/974))

- *(hl)* Add markup colors (#975) ([e1bd4389](https://github.com/rsvim/rsvim/commit/e1bd438989f53beed53293652c0f274d780e21c3)) by @linrongbin16 ([#975](https://github.com/rsvim/rsvim/pull/975))

- *(opt)* Add "fix-end-of-line" option (#998) ([e9ba928a](https://github.com/rsvim/rsvim/commit/e9ba928a76719155c9fb556774c469ce2cd87277)) by @linrongbin16 ([#998](https://github.com/rsvim/rsvim/pull/998))

- *(syn)* Load tree-sitter grammar/parser (#1041) ([500edd2a](https://github.com/rsvim/rsvim/commit/500edd2a0797e062d218d9b284de9a170384ac46)) by @linrongbin16 ([#1041](https://github.com/rsvim/rsvim/pull/1041))

- *(api)* Add "syntaxParserLibPath" option (#1046) ([748b3bed](https://github.com/rsvim/rsvim/commit/748b3bed5a5bc867dad4510eef9a049b1485a4f9)) by @linrongbin16 ([#1046](https://github.com/rsvim/rsvim/pull/1046))

- *(syn)* Add "loadTreeSitterParser" apis (#1056) ([1baaaf12](https://github.com/rsvim/rsvim/commit/1baaaf12f39a40552b1fa09d829ab3fbb59887c2)) by @linrongbin16 ([#1056](https://github.com/rsvim/rsvim/pull/1056))

- *(syn)* More apis for syntax (#1058) ([b7dd027a](https://github.com/rsvim/rsvim/commit/b7dd027aad3ff05dae7d6aa1017e22d546f7e9e3)) by @linrongbin16 ([#1058](https://github.com/rsvim/rsvim/pull/1058))

- *(fs)* Add "readFile" and "readTextFile" api (#1065) ([0a12d990](https://github.com/rsvim/rsvim/commit/0a12d9900e2467ada0636269dbd3c8873684f013)) by @linrongbin16 ([#1065](https://github.com/rsvim/rsvim/pull/1065))


### <!-- 1 -->Bug Fixes

- *(ui)* Fix missing whitespaces when "line-break" on (#197) ([0d1410e2](https://github.com/rsvim/rsvim/commit/0d1410e230780e64def866dc5968cd22c17cf57c)) by @linrongbin16 ([#197](https://github.com/rsvim/rsvim/pull/197))

- *(ui)* Fix viewport about display column of buffer chars (#223) ([1f80e0c2](https://github.com/rsvim/rsvim/commit/1f80e0c2b7ca2df94a4731fc6dbd1f85f14c0de0)) by @linrongbin16 ([#223](https://github.com/rsvim/rsvim/pull/223))

- *(ui)* Fix viewport with "display column" index instead of char index (#226) ([504f22b7](https://github.com/rsvim/rsvim/commit/504f22b793554fd57c22a175ecb611bfba2a9b86)) by @linrongbin16 ([#226](https://github.com/rsvim/rsvim/pull/226))

- *(cursor)* Fix tui cursor initialization for win (#319) ([65ebef96](https://github.com/rsvim/rsvim/commit/65ebef96089c7d6f1bafabfdf2ee76b577f92c18)) by @linrongbin16 ([#319](https://github.com/rsvim/rsvim/pull/319))

- *(cursor)* Fix cursor position at eol when move vertically (#321) ([1b63b313](https://github.com/rsvim/rsvim/commit/1b63b313625afb5fa5a4dc71c01aa9b4c23c39d7)) by @linrongbin16 ([#321](https://github.com/rsvim/rsvim/pull/321))

- *(cursor)* Fix cursor/viewport at buffer last line and window top/bottom (#333) ([0a6cae88](https://github.com/rsvim/rsvim/commit/0a6cae88624a825d4aaca3fb212d6ac9a82f0d96)) by @linrongbin16 ([#333](https://github.com/rsvim/rsvim/pull/333))

- *(miri)* Fix potential memory safety issue previously detected by cargo miri (#334) ([dcfd9bc6](https://github.com/rsvim/rsvim/commit/dcfd9bc6a0f5080a1f091cc11fe69d3715686066)) by @linrongbin16 ([#334](https://github.com/rsvim/rsvim/pull/334))

- *(viewport)* Fix rendering when start column>0 and wrap=false (#339) ([0b9629f1](https://github.com/rsvim/rsvim/commit/0b9629f1638de168be84f4cd38593b8aed8bb2ea)) by @linrongbin16 ([#339](https://github.com/rsvim/rsvim/pull/339))

- *(viewport)* Fix rendering when start column>0 and wrap=true (#340) ([49047ed5](https://github.com/rsvim/rsvim/commit/49047ed59974fc101dd5314c1790775f2c05d506)) by @linrongbin16 ([#340](https://github.com/rsvim/rsvim/pull/340))

- *(viewport)* Fix extra new line at the invisible eol char (#341) ([618f1751](https://github.com/rsvim/rsvim/commit/618f17511c426774ade6ac37524d03a78259bae2)) by @linrongbin16 ([#341](https://github.com/rsvim/rsvim/pull/341))

- *(cursor)* Fix CursorMoveTo/WindowScrollTo operation commands (#351) ([01ddb1b5](https://github.com/rsvim/rsvim/commit/01ddb1b55961ff3f5e6954f006c2aa8596c1357f)) by @linrongbin16 ([#351](https://github.com/rsvim/rsvim/pull/351))

- *(viewport)* Fix leftward/rightward anchor searching for super long line (#377) ([75a825cc](https://github.com/rsvim/rsvim/commit/75a825ccb2018be02437965eb27b83dc4bed72aa)) by @linrongbin16 ([#377](https://github.com/rsvim/rsvim/pull/377))

- *(viewport)* Fix super long word issues when 'linebreak=true' (#380) ([99b9390d](https://github.com/rsvim/rsvim/commit/99b9390da47073ca64e6a5bf1647ac471cf7cc3c)) by @linrongbin16 ([#380](https://github.com/rsvim/rsvim/pull/380))

- *(tui)* Recover terminal and save backtrace info when panic (#395) ([2299f0ef](https://github.com/rsvim/rsvim/commit/2299f0efdce62cc0013e5ae8bbc523d3698f9a6e)) by @linrongbin16 ([#395](https://github.com/rsvim/rsvim/pull/395))

- *(viewport)* Refactor wrap line anchor searching algorithms and fix edge cases (#407) ([f9c50ed6](https://github.com/rsvim/rsvim/commit/f9c50ed623d9bb97cb712b20e7f945e0e80f0387)) by @linrongbin16 ([#407](https://github.com/rsvim/rsvim/pull/407))

- *(cursor)* Limit cursor position by last visible char when back to normal mode (#438) ([b1225e8e](https://github.com/rsvim/rsvim/commit/b1225e8efd50ec41e0e31d4bbff4790dee1fd81c)) by @linrongbin16 ([#438](https://github.com/rsvim/rsvim/pull/438))

- *(cursor)* Fix cursor motion legacy boundary to work along anchor search (#440) ([a938add6](https://github.com/rsvim/rsvim/commit/a938add61daf2b0b58dacb64ed22e36a353db3cd)) by @linrongbin16 ([#440](https://github.com/rsvim/rsvim/pull/440))

- *(build)* Fix debug assertions warnings for release build (#446) ([68829ab2](https://github.com/rsvim/rsvim/commit/68829ab2610fc1e3d69d5f724d7178f1c10ed0f8)) by @linrongbin16 ([#446](https://github.com/rsvim/rsvim/pull/446))

- *(insert)* Fix non 1-width unicode chars insert (#452) ([2e2adb42](https://github.com/rsvim/rsvim/commit/2e2adb429177c09168ec409ef6d1f6020d164dda)) by @linrongbin16 ([#452](https://github.com/rsvim/rsvim/pull/452))

- *(cmdline)* Fix cmdline ex mode enter/leave (#496) ([ac8e5212](https://github.com/rsvim/rsvim/commit/ac8e5212099909ffabe52c63f16b88c8997e406c)) by @linrongbin16 ([#496](https://github.com/rsvim/rsvim/pull/496))

- *(cursor)* Fix cursor delete operation (#497) ([71fcfd73](https://github.com/rsvim/rsvim/commit/71fcfd7324cfb9a6af10e9402f9deeb4526782ef)) by @linrongbin16 ([#497](https://github.com/rsvim/rsvim/pull/497))

- *(cmdline)* Fix cmdline ex mode go back to normal mode (#499) ([787cfe3c](https://github.com/rsvim/rsvim/commit/787cfe3c8b1d59ef8a301c8f3fc43c6b03d216b8)) by @linrongbin16 ([#499](https://github.com/rsvim/rsvim/pull/499))

- *(cmdline)* Fix ex command editing (#503) ([569c50cf](https://github.com/rsvim/rsvim/commit/569c50cf16abe3c0d1a97920506f27a1130ef467)) by @linrongbin16 ([#503](https://github.com/rsvim/rsvim/pull/503))

- *(cursor)* Refactor cursor widget and fix cmdline indicator (":" "/" "?") (#507) ([4c3d1aa0](https://github.com/rsvim/rsvim/commit/4c3d1aa0e13dd7d9fc238338ad995f06018ecd84)) by @linrongbin16 ([#507](https://github.com/rsvim/rsvim/pull/507))

- *(ascii)* Fix ASCII control code display (#512) ([71ebfd8b](https://github.com/rsvim/rsvim/commit/71ebfd8b5c41f2e6df91d4c48d2e547d912b77bf)) by @linrongbin16 ([#512](https://github.com/rsvim/rsvim/pull/512))

- *(eol)* Correctly detect all platform line breaks (LF/CR/CRLF) (#517) ([03cad4e7](https://github.com/rsvim/rsvim/commit/03cad4e75fbd91aabfe1785ea58b8a9cbb46ea40)) by @linrongbin16 ([#517](https://github.com/rsvim/rsvim/pull/517))

- *(unicode)* Fix unicode character width detection for some special chars (#538) ([1543fac6](https://github.com/rsvim/rsvim/commit/1543fac60ca2df07aa7f54637f38df83432fefd5)) by @linrongbin16 ([#538](https://github.com/rsvim/rsvim/pull/538))

- *(unicode)* Fix logic cells canvas drawing for >1 width unicode characters (#539) ([5143c342](https://github.com/rsvim/rsvim/commit/5143c342b426361c91bc20666a79335a56dc38f9)) by @linrongbin16 ([#539](https://github.com/rsvim/rsvim/pull/539))

- *(cursor)* Fix cursor motion jumps when "wrap=false" (#550) ([8e252d9c](https://github.com/rsvim/rsvim/commit/8e252d9cd24521e650a7e06cde7d83c8cb6bd817)) by @linrongbin16 ([#550](https://github.com/rsvim/rsvim/pull/550))

- *(loader)* Fix module fs loader and add tests for "fetch_module_tree" (#554) ([42bfef47](https://github.com/rsvim/rsvim/commit/42bfef474d3100461407723f8785f4e989b03a70)) by @linrongbin16 ([#554](https://github.com/rsvim/rsvim/pull/554))

- *(timeout)* Skip 'setTimeout' callback execution if cancelled (#560) ([e4151321](https://github.com/rsvim/rsvim/commit/e4151321ab63d441866b9a841a5d22cd9fe5659b)) by @linrongbin16 ([#560](https://github.com/rsvim/rsvim/pull/560))

- *(api)* Returns i32 integer for setTimeout api (#561) ([c84fed02](https://github.com/rsvim/rsvim/commit/c84fed026bd7127e809fc305400038504a354242)) by @linrongbin16 ([#561](https://github.com/rsvim/rsvim/pull/561))

- *(log)* Only create file log when log level >= info (#569) ([31bbe8a5](https://github.com/rsvim/rsvim/commit/31bbe8a55159ad4bafebb1a0581e01daefa7277b)) by @linrongbin16 ([#569](https://github.com/rsvim/rsvim/pull/569))

- *(log)* Fix file log initialization (#572) ([f74f6d12](https://github.com/rsvim/rsvim/commit/f74f6d12c0ff7ed7075f9c484aafaf0f6f6b00b4)) by @linrongbin16 ([#572](https://github.com/rsvim/rsvim/pull/572))

- *(insert)* Refactor "CursorInsert" op and insert tab (#580) ([bdba9775](https://github.com/rsvim/rsvim/commit/bdba9775e69a12de519181222a0dfeae0cf574ad)) by @linrongbin16 ([#580](https://github.com/rsvim/rsvim/pull/580))

- *(cmdline)* Simplify/fix the command-line input/message switching logic (#590) ([ab3d004a](https://github.com/rsvim/rsvim/commit/ab3d004ac6fe3209b14a4b31c5e5eab94b5f72b0)) by @linrongbin16 ([#590](https://github.com/rsvim/rsvim/pull/590))

- *(js)* Handle invalid expressions in `:js` command (#603) ([0b1a72d3](https://github.com/rsvim/rsvim/commit/0b1a72d36a5130b794124f58c930130dc362e63d)) by @linrongbin16 ([#603](https://github.com/rsvim/rsvim/pull/603))

- *(canvas)* Avoid cursor twinkling while flushing canvas shaders for windows terminal (#607) ([92b165f8](https://github.com/rsvim/rsvim/commit/92b165f87f1e3781ce2d4b5736308b3ae80224c7)) by @linrongbin16 ([#607](https://github.com/rsvim/rsvim/pull/607))

- *(js)* Fix pending imports counting (#661) ([00b00e71](https://github.com/rsvim/rsvim/commit/00b00e71c16c1fb9bf50e23f55c12d0b3275dbcc)) by @linrongbin16 ([#661](https://github.com/rsvim/rsvim/pull/661))

- *(js)* Fix panic when failed to resolve modules in static imports (#671) ([d9b30caa](https://github.com/rsvim/rsvim/commit/d9b30caa4157cdbbe548fba1b708cf9f65e88443)) by @linrongbin16 ([#671](https://github.com/rsvim/rsvim/pull/671))

- *(module)* Fix es module resolution (#683) ([d09dd17d](https://github.com/rsvim/rsvim/commit/d09dd17dcdba507ad27cdf2164c1e890502f4078)) by @linrongbin16 ([#683](https://github.com/rsvim/rsvim/pull/683))

- *(js)* Fix module evaluation thrown exceptions (#688) ([936a8716](https://github.com/rsvim/rsvim/commit/936a871610568885d42263bd502b44855d64ca1d)) by @linrongbin16 ([#688](https://github.com/rsvim/rsvim/pull/688))

- *(task)* Track missing detach-able tasks (#696) ([f696d299](https://github.com/rsvim/rsvim/commit/f696d29920119eafb6a70283b2657c4e9cc12380)) by @linrongbin16 ([#696](https://github.com/rsvim/rsvim/pull/696))

- *(message)* Append waiting messages to message history (#736) ([2f06414b](https://github.com/rsvim/rsvim/commit/2f06414ba353106dd7fc7793d175be0233fb11af)) by @linrongbin16 ([#736](https://github.com/rsvim/rsvim/pull/736))

- *(cmd)* Allow command calls async/await api (#758) ([39d41637](https://github.com/rsvim/rsvim/commit/39d41637cf931623dcc7ed881c02359ecc00d601)) by @linrongbin16 ([#758](https://github.com/rsvim/rsvim/pull/758))

- *(api)* Refactor typescript configs and fix Float16Array type (#759) ([bdb509c1](https://github.com/rsvim/rsvim/commit/bdb509c1a519a8f659617e87086addb866a7aacd)) by @linrongbin16 ([#759](https://github.com/rsvim/rsvim/pull/759))

- *(ui)* Fix relative position/size of sub-components in command-line (#816) ([7020dfe2](https://github.com/rsvim/rsvim/commit/7020dfe2a8a1e79032fd86627ea28d32b49a7539)) by @linrongbin16 ([#816](https://github.com/rsvim/rsvim/pull/816))

- *(ui)* Fix default window and command-line layout (#817) ([8d2c54bb](https://github.com/rsvim/rsvim/commit/8d2c54bb4d82f33c1eb91f43b29f2de0521ec0a3)) by @linrongbin16 ([#817](https://github.com/rsvim/rsvim/pull/817))

- *(undo)* Record cursor positions before and after editing operations (#868) ([19e04fc9](https://github.com/rsvim/rsvim/commit/19e04fc95a825b627d6e19e08a79f5ac29dee241)) by @linrongbin16 ([#868](https://github.com/rsvim/rsvim/pull/868))

- *(syn)* Trigger next syntax parsing immediately if has more pendings (#890) ([fc655b49](https://github.com/rsvim/rsvim/commit/fc655b49246149b50256e651adb5f4e4f723898f)) by @linrongbin16 ([#890](https://github.com/rsvim/rsvim/pull/890))

- *(syn)* Fix syntax highlight query duplicate start point (#949) ([e51c705e](https://github.com/rsvim/rsvim/commit/e51c705e3df19bbe06178472e200ff7b02bed0d8)) by @linrongbin16 ([#949](https://github.com/rsvim/rsvim/pull/949))

- *(hl)* Render ui foreground/background for empty spaces (#967) ([886b6502](https://github.com/rsvim/rsvim/commit/886b6502d00f225fcfef6f40b56dc196308c7282)) by @linrongbin16 ([#967](https://github.com/rsvim/rsvim/pull/967))

- *(ui)* Refactor canvas cells fill (#971) ([f37accbd](https://github.com/rsvim/rsvim/commit/f37accbdb9e9cbd9574486aaddccdc2bcdb37ed7)) by @linrongbin16 ([#971](https://github.com/rsvim/rsvim/pull/971))

- *(ui)* Fix viewport search (#992) ([58a7ac61](https://github.com/rsvim/rsvim/commit/58a7ac619ec1227e454890725ce8a17c6a47251d)) by @linrongbin16 ([#992](https://github.com/rsvim/rsvim/pull/992))

- *(ui)* Fix cmdline colorscheme (#1011) ([539bdce3](https://github.com/rsvim/rsvim/commit/539bdce3c26a64cdd00091680e081830597096f4)) by @linrongbin16 ([#1011](https://github.com/rsvim/rsvim/pull/1011))


### <!-- 2 -->Performance Improvements

- *(start)* Make snapshot v3 (#199) ([2f04e781](https://github.com/rsvim/rsvim/commit/2f04e7810b300937e1276f567764426a1c5eb69b)) by @linrongbin16 ([#199](https://github.com/rsvim/rsvim/pull/199))

- *(start)* Initialize built-in modules with snapshot (#205) ([73a92c0c](https://github.com/rsvim/rsvim/commit/73a92c0c3d20ac7c66a184987ce4024ad4918c7c)) by @linrongbin16 ([#205](https://github.com/rsvim/rsvim/pull/205))

- *(start)* Compress snapshot blob (#206) ([ac8420d1](https://github.com/rsvim/rsvim/commit/ac8420d17605aef8f975fdb5000774017a9f0384)) by @linrongbin16 ([#206](https://github.com/rsvim/rsvim/pull/206))

- *(start)* Move built-in runtime modules evaluation to snapshot phase (#211) ([8003a3e4](https://github.com/rsvim/rsvim/commit/8003a3e47153ea1e6fc208c8934eb256c2fe119f)) by @linrongbin16 ([#211](https://github.com/rsvim/rsvim/pull/211))

- *(hash)* Use ahash instead of std lib (#236) ([c5d58b66](https://github.com/rsvim/rsvim/commit/c5d58b6679fb09cfbeae9ba4dd9e12ad352ab307)) by @linrongbin16 ([#236](https://github.com/rsvim/rsvim/pull/236))

- *(buf)* Cache line-wise chars index and display width (#303) ([b97c52e1](https://github.com/rsvim/rsvim/commit/b97c52e18b28fbb71e23a73da6d098fbc3a69075)) by @linrongbin16 ([#303](https://github.com/rsvim/rsvim/pull/303))

- *(cursor)* Reduce lock in moving cursor and scrolling window motion (#352) ([3935cc8b](https://github.com/rsvim/rsvim/commit/3935cc8b145ea26b4f6a6115b7d43288cc16cf6a)) by @linrongbin16 ([#352](https://github.com/rsvim/rsvim/pull/352))

- *(cursor)* Reduce 1 lock/unlock call during cursor movement (#383) ([ce0dbfd8](https://github.com/rsvim/rsvim/commit/ce0dbfd8d20bcb405682c80828dd1be6e9f1bba9)) by @linrongbin16 ([#383](https://github.com/rsvim/rsvim/pull/383))

- *(cursor)* Drop duplicated normalization between window scroll "to" and "by" (#436) ([7543c731](https://github.com/rsvim/rsvim/commit/7543c73134b9ed840b5b883963aa99b2dd76be4a)) by @linrongbin16 ([#436](https://github.com/rsvim/rsvim/pull/436))

- *(viewport)* More compact memory layout (#441) ([4bf887fb](https://github.com/rsvim/rsvim/commit/4bf887fba055b49743a9230ed7ddeeb5c2f34ad1)) by @linrongbin16 ([#441](https://github.com/rsvim/rsvim/pull/441))

- *(viewport)* Drop unnecessary locks on readyonly viewport (#443) ([b6b91832](https://github.com/rsvim/rsvim/commit/b6b91832d900775c538fdda905d4724e3e51de31)) by @linrongbin16 ([#443](https://github.com/rsvim/rsvim/pull/443))

- *(canvas)* Use "clone_from" when cloning frame (#511) ([61c7500b](https://github.com/rsvim/rsvim/commit/61c7500b48dfd08f07d2b11356b68a1a6d3bb2a9)) by @linrongbin16 ([#511](https://github.com/rsvim/rsvim/pull/511))

- *(cli)* Move to "fern" to reduce binary size (#567) ([fde0bb33](https://github.com/rsvim/rsvim/commit/fde0bb3384f1e706403126e024c2ffdd1a61ecd1)) by @linrongbin16 ([#567](https://github.com/rsvim/rsvim/pull/567))

- *(cli)* Move to "lexopt" to reduce binary size (#568) ([677005dc](https://github.com/rsvim/rsvim/commit/677005dc5f4b08bacc4c447ea592ffa8100f2be3)) by @linrongbin16 ([#568](https://github.com/rsvim/rsvim/pull/568))

- *(js)* Avoid duplicate loop tick when waiting for dynamic import loaders (#654) ([2e92d017](https://github.com/rsvim/rsvim/commit/2e92d017211c8a7884fe7ccd7783ce2714d6cc74)) by @linrongbin16 ([#654](https://github.com/rsvim/rsvim/pull/654))

- *(alloc)* Add optional custom allocator (#663) ([288e35b7](https://github.com/rsvim/rsvim/commit/288e35b76c0424c2fba7833d0e212635d5b2db48)) by @linrongbin16 ([#663](https://github.com/rsvim/rsvim/pull/663))

- *(opts)* Reduce internal options data types (#714) ([34ed2d7e](https://github.com/rsvim/rsvim/commit/34ed2d7eab57e566faa979d5050f27deef8bc58f)) by @linrongbin16 ([#714](https://github.com/rsvim/rsvim/pull/714))

- *(text)* Cache cloned lines with LRU + Rc<String> (#716) ([a103f711](https://github.com/rsvim/rsvim/commit/a103f7116a1f4c2f63c2e88624b8e4a97334f617)) by @linrongbin16 ([#716](https://github.com/rsvim/rsvim/pull/716))

- *(msg)* Drain master/js messages to reduce memory allocation (#718) ([1b994234](https://github.com/rsvim/rsvim/commit/1b9942349a18b51b005d55f561963b4615e0661d)) by @linrongbin16 ([#718](https://github.com/rsvim/rsvim/pull/718))

- *(js)* Improve module linear lookup with hash map (#734) ([31fbb689](https://github.com/rsvim/rsvim/commit/31fbb6897846af78584d4305673598b07828e948)) by @linrongbin16 ([#734](https://github.com/rsvim/rsvim/pull/734))

- *(buf)* Revert back to "lru" cache instead of "clru" (#735) ([91d58876](https://github.com/rsvim/rsvim/commit/91d5887615aaaf47fcd42937276774614dc48f96)) by @linrongbin16 ([#735](https://github.com/rsvim/rsvim/pull/735))

- *(ui)* Avoid too many Vector allocations in window drawing (#968) ([72f8dfc5](https://github.com/rsvim/rsvim/commit/72f8dfc5b6374979cab7e3c7d824b9c7f5a85283)) by @linrongbin16 ([#968](https://github.com/rsvim/rsvim/pull/968))

- *(ui)* Reduce Vector allocation in canvas (#970) ([c4c404cd](https://github.com/rsvim/rsvim/commit/c4c404cd0f24ec08b765954cbb462a0a6708322a)) by @linrongbin16 ([#970](https://github.com/rsvim/rsvim/pull/970))

- *(ui)* Reduce vec allocation in canvas shader (#972) ([76a0a9fb](https://github.com/rsvim/rsvim/commit/76a0a9fba14381569f64de9ab0b2537c9766f52c)) by @linrongbin16 ([#972](https://github.com/rsvim/rsvim/pull/972))

- *(ui)* Detect dirty cells with range based bitmap (#978) ([61c751b2](https://github.com/rsvim/rsvim/commit/61c751b2f3a0af724398695933533fd5dab96497)) by @linrongbin16 ([#978](https://github.com/rsvim/rsvim/pull/978))

- *(lock)* Switch to std mutex (#997) ([866e8794](https://github.com/rsvim/rsvim/commit/866e87940b4303b3dcafb4f463c41a358daf238c)) by @linrongbin16 ([#997](https://github.com/rsvim/rsvim/pull/997))

- *(ui)* Bring back binary search on words when "linebreak=true" (#1010) ([83b7038c](https://github.com/rsvim/rsvim/commit/83b7038cc17bff596e70f048665ba3f1d2b100e4)) by @linrongbin16 ([#1010](https://github.com/rsvim/rsvim/pull/1010))

- *(ui)* Cache rope query result to avoid duplicate calculation (#1007) ([ae381544](https://github.com/rsvim/rsvim/commit/ae381544700a29308f017768df8e9870ad7e84a6)) by @linrongbin16 ([#1007](https://github.com/rsvim/rsvim/pull/1007))

- *(ui)* Reuse canvas shader buffers (#1028) ([7ca6940e](https://github.com/rsvim/rsvim/commit/7ca6940e8f3bd7858cbb1f79579c93a2ae61fcd2)) by @linrongbin16 ([#1028](https://github.com/rsvim/rsvim/pull/1028))

- *(js)* Minify runtime api library (#1045) ([dfc416e9](https://github.com/rsvim/rsvim/commit/dfc416e98e6f7d826d9859a30b6a91294453e7a9)) by @linrongbin16 ([#1045](https://github.com/rsvim/rsvim/pull/1045))


### <!-- 3 -->Code Refactoring

- *(workspace)* Manage with workspace (#183) ([0da0e170](https://github.com/rsvim/rsvim/commit/0da0e17043b4bec3e6403839aba90661c59c4631)) by @linrongbin16 ([#183](https://github.com/rsvim/rsvim/pull/183))

- *(cli)* Remove "Cargo.toml" from version detect (#186) ([d550dbc4](https://github.com/rsvim/rsvim/commit/d550dbc4a98793e604724c4f935626ec7be8908f)) by @linrongbin16 ([#186](https://github.com/rsvim/rsvim/pull/186))

- *(js)* Merge built-in modules init into constructor (#194) ([52b3b022](https://github.com/rsvim/rsvim/commit/52b3b022d75becb238530702897584f696e29165)) by @linrongbin16 ([#194](https://github.com/rsvim/rsvim/pull/194))

- *(js)* Add basic v8 methods (#195) ([c9bde990](https://github.com/rsvim/rsvim/commit/c9bde990c00472e5b7157d15559f0b43a1148ff8)) by @linrongbin16 ([#195](https://github.com/rsvim/rsvim/pull/195))

- *(js)* Simplify built-in module as single file (#200) ([ec414499](https://github.com/rsvim/rsvim/commit/ec41449959907f7425e1aa6b5fcf67e83252634e)) by @linrongbin16 ([#200](https://github.com/rsvim/rsvim/pull/200))

- *(js)* Refactor context data index (#202) ([b15e932d](https://github.com/rsvim/rsvim/commit/b15e932df295be606ffd6ed0428ea77b850effc8)) by @linrongbin16 ([#202](https://github.com/rsvim/rsvim/pull/202))

- *(js)* Clean up not used code (#215) ([1e0e0351](https://github.com/rsvim/rsvim/commit/1e0e0351f8b732299a34f365f2a190f6dc2ce873)) by @linrongbin16 ([#215](https://github.com/rsvim/rsvim/pull/215))

- *(cli)* Adjust initialize sequence in main loop (#215) ([1e0e0351](https://github.com/rsvim/rsvim/commit/1e0e0351f8b732299a34f365f2a190f6dc2ce873)) by @linrongbin16 ([#215](https://github.com/rsvim/rsvim/pull/215))

- *(ui)* Rename module "tree::util" to "tree::ptr" (#215) ([1e0e0351](https://github.com/rsvim/rsvim/commit/1e0e0351f8b732299a34f365f2a190f6dc2ce873)) by @linrongbin16 ([#215](https://github.com/rsvim/rsvim/pull/215))

- *(env)* Rename glovar to envar (#216) ([9744c693](https://github.com/rsvim/rsvim/commit/9744c693acf3fe0ce012c39f1d9dee4cebc46b82)) by @linrongbin16 ([#216](https://github.com/rsvim/rsvim/pull/216))

- *(ui)* Add "sync_from_top_left" and other APIs (#227) ([f1529186](https://github.com/rsvim/rsvim/commit/f15291861c120a45feae1185785b06047ae19a99)) by @linrongbin16 ([#227](https://github.com/rsvim/rsvim/pull/227))

- *(ui)* Migrate window content rendering to viewport (#228) ([697ed5a0](https://github.com/rsvim/rsvim/commit/697ed5a0965fc62c5969cef1249ecaf00ec19fac)) by @linrongbin16 ([#228](https://github.com/rsvim/rsvim/pull/228))

- *(buf)* Add line-wise char/column index (#253) ([826d77f6](https://github.com/rsvim/rsvim/commit/826d77f6371f8e6b61aa74a105abe04c8ecaf094)) by @linrongbin16 ([#253](https://github.com/rsvim/rsvim/pull/253))

- *(macros)* Embed "$crate" in macros to remove extra "use" dependencies (#271) ([33709d9c](https://github.com/rsvim/rsvim/commit/33709d9c6cb5e76596df8dfe11036a0296efa1ca)) by @linrongbin16 ([#271](https://github.com/rsvim/rsvim/pull/271))

- *(cursor)* Rename multiple struct names to avoid confliction (#273) ([a17eb56b](https://github.com/rsvim/rsvim/commit/a17eb56b784a603afdea58e7174fe926da6c7408)) by @linrongbin16 ([#273](https://github.com/rsvim/rsvim/pull/273))

- *(cursor)* Small refactors for cursor style (#276) ([f8adb3c9](https://github.com/rsvim/rsvim/commit/f8adb3c99e7304aec39f483a906c64ab63ea3d5d)) by @linrongbin16 ([#276](https://github.com/rsvim/rsvim/pull/276))

- *(coord)* Rename module 'cart' to 'coord' and import all types with '*' (#288) ([9534e5ae](https://github.com/rsvim/rsvim/commit/9534e5ae43f244482fcaffd1f9ac505e25fc7a2e)) by @linrongbin16 ([#288](https://github.com/rsvim/rsvim/pull/288))

- *(opts)* Refactor global/local/global-local options (#290) ([0647d4a9](https://github.com/rsvim/rsvim/commit/0647d4a9cd3b81141eee4e60ba598fe140424fc1)) by @linrongbin16 ([#290](https://github.com/rsvim/rsvim/pull/290))

- *(config)* Migrate crate 'directories' to 'dirs' (#291) ([828eba92](https://github.com/rsvim/rsvim/commit/828eba92d9c0b11269a7524143541fb100b8efd8)) by @linrongbin16 ([#291](https://github.com/rsvim/rsvim/pull/291))

- *(mod)* Refactor pub mod for easier imports (#293) ([790f2452](https://github.com/rsvim/rsvim/commit/790f2452aeee097c081b4b53749feee0ca1f3451)) by @linrongbin16 ([#293](https://github.com/rsvim/rsvim/pull/293))

- *(log)* Rename log envfilter to 'RSVIM_LOG' (#295) ([0e03f2f0](https://github.com/rsvim/rsvim/commit/0e03f2f0cad3462160ed2932f2e87fd2c1ca6a6b)) by @linrongbin16 ([#295](https://github.com/rsvim/rsvim/pull/295))

- *(prelude)* Add prelude and refactor other misc (#298) ([b4a8ab7b](https://github.com/rsvim/rsvim/commit/b4a8ab7beaf99cb6e6ce5b861763d7082ff6d0ed)) by @linrongbin16 ([#298](https://github.com/rsvim/rsvim/pull/298))

- *(prelude)* Pub use `geo` crate (#300) ([148e934b](https://github.com/rsvim/rsvim/commit/148e934b881bfa990512336b731dcd2e673e6fec)) by @linrongbin16 ([#300](https://github.com/rsvim/rsvim/pull/300))

- *(opts)* Simplify options builder with derived_builder (#302) ([79e4844a](https://github.com/rsvim/rsvim/commit/79e4844a82dee1461036bc355275d36edadb2165)) by @linrongbin16 ([#302](https://github.com/rsvim/rsvim/pull/302))

- *(buf)* Add 'resize_cached_lines' api for buffer cache (#305) ([8ef0d9e8](https://github.com/rsvim/rsvim/commit/8ef0d9e888def523968fc245ea5202032b50f287)) by @linrongbin16 ([#305](https://github.com/rsvim/rsvim/pull/305))

- *(lock)* Simplify lock definitions (#313) ([ec242f6e](https://github.com/rsvim/rsvim/commit/ec242f6ed91d1a1466c7e428c3c6eda6e696da3c)) by @linrongbin16 ([#313](https://github.com/rsvim/rsvim/pull/313))

- *(viewport)* Refactor viewport and fix potential memory safety (#328) ([08bb79fb](https://github.com/rsvim/rsvim/commit/08bb79fb46e598fda8b9f0a9e7df380ff9a9f080)) by @linrongbin16 ([#328](https://github.com/rsvim/rsvim/pull/328))

- *(debug)* Migrate 'assert' to 'debug_assert' (#330) ([718d776b](https://github.com/rsvim/rsvim/commit/718d776b20b0af6d157e6aedb20a47b664ec02d9)) by @linrongbin16 ([#330](https://github.com/rsvim/rsvim/pull/330))

- *(viewport)* Extract line-wise viewport calculation to single function (#331) ([d1a862b2](https://github.com/rsvim/rsvim/commit/d1a862b2aad0ffc012873a58da5cefb4c50ec048)) by @linrongbin16 ([#331](https://github.com/rsvim/rsvim/pull/331))

- *(lock)* Mirgrate 'RwLock' to 'Mutex' (#336) ([43990f31](https://github.com/rsvim/rsvim/commit/43990f31aa07d1d05ee8e6c3cd6790d45e9fde7b)) by @linrongbin16 ([#336](https://github.com/rsvim/rsvim/pull/336))

- *(viewport)* Remove 'unsafe' code block with rc/refcell (#337) ([cc163c76](https://github.com/rsvim/rsvim/commit/cc163c76a5a04e2f1fcf47732d7bb426e0c39677)) by @linrongbin16 ([#337](https://github.com/rsvim/rsvim/pull/337))

- *(ui)* Rewrite ui tree and remove 'unsfe' code block (#338) ([c56eafcc](https://github.com/rsvim/rsvim/commit/c56eafcc22ee751c34ba376627a9e2d520ccc3df)) by @linrongbin16 ([#338](https://github.com/rsvim/rsvim/pull/338))

- *(buf)* Add 'last_visible_char_on_line_since' api (#345) ([87bf8bf0](https://github.com/rsvim/rsvim/commit/87bf8bf055fc80bde7b43097426e7091c0343c1f)) by @linrongbin16 ([#345](https://github.com/rsvim/rsvim/pull/345))

- *(cursor)* Refactor editor operations command (#346) ([0000b849](https://github.com/rsvim/rsvim/commit/0000b849b10c396062118d1619648937db128925)) by @linrongbin16 ([#346](https://github.com/rsvim/rsvim/pull/346))

- *(cursor)* Add more test cases for cursor movement (#349) ([36ed4ba8](https://github.com/rsvim/rsvim/commit/36ed4ba8d7b7aef151a03d9b6f1ccf56b585914e)) by @linrongbin16 ([#349](https://github.com/rsvim/rsvim/pull/349))

- *(cursor)* Enable 2D window scroll in a single command (#350) ([a013738f](https://github.com/rsvim/rsvim/commit/a013738f3e66274bd823f68ab1a75e4eb69b0923)) by @linrongbin16 ([#350](https://github.com/rsvim/rsvim/pull/350))

- *(cursor)* Refactor operation command conversions (#353) ([4628f0ad](https://github.com/rsvim/rsvim/commit/4628f0ad03bc4a5480b24fa80de7fede27ea30f7)) by @linrongbin16 ([#353](https://github.com/rsvim/rsvim/pull/353))

- *(cursor)* Rename `x`, `y` variable names to lines/chars/columns for readability (#362) ([d1b5eff7](https://github.com/rsvim/rsvim/commit/d1b5eff7db34befcf31442ff31b76cabbf4c23b8)) by @linrongbin16 ([#362](https://github.com/rsvim/rsvim/pull/362))

- *(viewport)* Export low-level line wise viewport calculation api (#363) ([28daeb02](https://github.com/rsvim/rsvim/commit/28daeb026087f68467c63d066df379a5dabb9392)) by @linrongbin16 ([#363](https://github.com/rsvim/rsvim/pull/363))

- *(viewport)* Refactor internal viewport revert search functions (#370) ([7deaeb04](https://github.com/rsvim/rsvim/commit/7deaeb04cde3cd76bcfa30331da4c6908338f913)) by @linrongbin16 ([#370](https://github.com/rsvim/rsvim/pull/370))

- *(viewport)* Refactor internal viewport revert searching functions 2 (#371) ([8d3e4b3e](https://github.com/rsvim/rsvim/commit/8d3e4b3e9636e4d6e0c4e7be968ce42756952887)) by @linrongbin16 ([#371](https://github.com/rsvim/rsvim/pull/371))

- *(buf)* Improve allocation for column index (#372) ([5153dd0d](https://github.com/rsvim/rsvim/commit/5153dd0d3fbbdff205491a032c4211c4b02b149c)) by @linrongbin16 ([#372](https://github.com/rsvim/rsvim/pull/372))

- *(viewport)* Use 'Option' as function result for better readibility (#376) ([0d620df3](https://github.com/rsvim/rsvim/commit/0d620df35d8d5170711a6b0783a64544f629d21c)) by @linrongbin16 ([#376](https://github.com/rsvim/rsvim/pull/376))

- *(viewport)* More practical algorithm for 'linebreak=true' (#379) ([d48a9ea4](https://github.com/rsvim/rsvim/commit/d48a9ea40b77b82950efba3db063e8899caf5460)) by @linrongbin16 ([#379](https://github.com/rsvim/rsvim/pull/379))

- *(cursor)* Extract duplicated code logic into single method (#384) ([8be594e5](https://github.com/rsvim/rsvim/commit/8be594e531fc69e0dfaf1740ba464a2c3dfbd85c)) by @linrongbin16 ([#384](https://github.com/rsvim/rsvim/pull/384))

- *(viewport)* Reduce duplicated logic with line-wise func arg (#397) ([18badf90](https://github.com/rsvim/rsvim/commit/18badf908d2101ace70e97ae7717c1419d922342)) by @linrongbin16 ([#397](https://github.com/rsvim/rsvim/pull/397))

- *(viewport)* Refactor internal leftward/rightward searching api (#399) ([0ce7f9e3](https://github.com/rsvim/rsvim/commit/0ce7f9e3dbe2628e8792527cccc06be7d159b1e1)) by @linrongbin16 ([#399](https://github.com/rsvim/rsvim/pull/399))

- *(buf)* Refactor buffer's last char apis (#403) ([57fc324a](https://github.com/rsvim/rsvim/commit/57fc324a34553a472e79b1f952a756546c7a1dbe)) by @linrongbin16 ([#403](https://github.com/rsvim/rsvim/pull/403))

- *(buf)* Rename last char on line api (#405) ([77d74901](https://github.com/rsvim/rsvim/commit/77d749012babbc9545f4e69cc50b22d5e768295a)) by @linrongbin16 ([#405](https://github.com/rsvim/rsvim/pull/405))

- *(viewport)* Simplify the "cannot fully contain target line" logic (#414) ([49554168](https://github.com/rsvim/rsvim/commit/495541680d724c171f00611511894842d6ce2d03)) by @linrongbin16 ([#414](https://github.com/rsvim/rsvim/pull/414))

- *(cursor)* Refactor cursor motion in normal mode (#433) ([f3a6ef50](https://github.com/rsvim/rsvim/commit/f3a6ef50f272075558f965f3e4098f346e1fb673)) by @linrongbin16 ([#433](https://github.com/rsvim/rsvim/pull/433))

- *(state)* Add "handle_op" api for low-level operation layer (#447) ([429f8cc4](https://github.com/rsvim/rsvim/commit/429f8cc43f676cce1a548ac085651d95d7b1133d)) by @linrongbin16 ([#447](https://github.com/rsvim/rsvim/pull/447))

- *(viewport)* Use viewport as a fundamental typeset module (#456) ([5e7e3186](https://github.com/rsvim/rsvim/commit/5e7e3186bf25e590235d30cd564443189ef48f33)) by @linrongbin16 ([#456](https://github.com/rsvim/rsvim/pull/456))

- *(buf)* Extract internal rope out from buffer as a text content backend (#458) ([309dfd8d](https://github.com/rsvim/rsvim/commit/309dfd8d026d7dbeeb081928e927350cc47190f2)) by @linrongbin16 ([#458](https://github.com/rsvim/rsvim/pull/458))

- *(buf)* Shorten the 'rope' api name (#459) ([89e727dd](https://github.com/rsvim/rsvim/commit/89e727dde8f6d47122aeacd7749ea7f014eb2f09)) by @linrongbin16 ([#459](https://github.com/rsvim/rsvim/pull/459))

- *(viewport)* Move parameter from 'buffer' to 'text' in viewport api (#461) ([e24fc16d](https://github.com/rsvim/rsvim/commit/e24fc16d38b5ae33b2b5b96f67c954016e1a5282)) by @linrongbin16 ([#461](https://github.com/rsvim/rsvim/pull/461))

- *(buf)* Drop bypassed text api in buffer (#463) ([847edb00](https://github.com/rsvim/rsvim/commit/847edb00f8a417c69e8fb56df4c850cd8bae066a)) by @linrongbin16 ([#463](https://github.com/rsvim/rsvim/pull/463))

- *(cmdline)* Save current window id when cursor moves to the cmdline (#472) ([8f53329c](https://github.com/rsvim/rsvim/commit/8f53329c37b90280fb0b82e42b302f522bed125b)) by @linrongbin16 ([#472](https://github.com/rsvim/rsvim/pull/472))

- *(cursor)* Extract common pattern into 'cursor_move' operation (#487) ([f64510d5](https://github.com/rsvim/rsvim/commit/f64510d540d9c87b09b397504e6c3abf6ce31aba)) by @linrongbin16 ([#487](https://github.com/rsvim/rsvim/pull/487))

- *(cursor)* Extract common pattern into 'cursor_insert' operation (#488) ([dbde73cf](https://github.com/rsvim/rsvim/commit/dbde73cf739b32980ec62640fa867e9548c22a07)) by @linrongbin16 ([#488](https://github.com/rsvim/rsvim/pull/488))

- *(cursor)* Extract common patterns into 'cursor_delete' operation (#489) ([25c9dd99](https://github.com/rsvim/rsvim/commit/25c9dd99c9e2f2fbdf5ead615ff31758b8981cb4)) by @linrongbin16 ([#489](https://github.com/rsvim/rsvim/pull/489))

- *(cursor)* Merge cursor move/edit operations (#490) ([d8f2d0ce](https://github.com/rsvim/rsvim/commit/d8f2d0ce6edf9e8e2122b6d3c65e1a84100c2422)) by @linrongbin16 ([#490](https://github.com/rsvim/rsvim/pull/490))

- *(cursor)* Use 'cursor_insert' operation when 'goto command line ex mode' (#491) ([34357691](https://github.com/rsvim/rsvim/commit/343576913f52adf0cb45dfc567bc50f725489b00)) by @linrongbin16 ([#491](https://github.com/rsvim/rsvim/pull/491))

- *(cursor)* Move insert into 'Text' and hide cache details (#492) ([05e0df56](https://github.com/rsvim/rsvim/commit/05e0df569ce783acbcf8ccf557702911817264c3)) by @linrongbin16 ([#492](https://github.com/rsvim/rsvim/pull/492))

- *(cursor)* Move delete operation into 'Text' to hide cache details (#493) ([410de847](https://github.com/rsvim/rsvim/commit/410de8473475bb0d357e7ea55783ae256e4c632d)) by @linrongbin16 ([#493](https://github.com/rsvim/rsvim/pull/493))

- *(cursor)* Refine cursor operation api return value (#498) ([84e0e931](https://github.com/rsvim/rsvim/commit/84e0e9317a1d7180d6690b8493269d825e5ee50b)) by @linrongbin16 ([#498](https://github.com/rsvim/rsvim/pull/498))

- *(inode)* Use 'enum_dispatch' to reduce duplicated code (#500) ([0b4af6f8](https://github.com/rsvim/rsvim/commit/0b4af6f87cce45b8d7e9665c068c1c719ddde0b6)) by @linrongbin16 ([#500](https://github.com/rsvim/rsvim/pull/500))

- *(enum)* Dispatch trait enums with 'enum_dispatch' (#501) ([7e5a88ca](https://github.com/rsvim/rsvim/commit/7e5a88ca4c02170290a851cd17b2133635e7d501)) by @linrongbin16 ([#501](https://github.com/rsvim/rsvim/pull/501))

- *(enum)* Impl enum dispatcher for 'Widgetable' and remove crate (#506) ([7c72203c](https://github.com/rsvim/rsvim/commit/7c72203c11bb4b1b9098b7dce5b6e53dc861caa9)) by @linrongbin16 ([#506](https://github.com/rsvim/rsvim/pull/506))

- *(eol)* Shorten 'empty_eol' to 'eol' (#516) ([ddbbcbf7](https://github.com/rsvim/rsvim/commit/ddbbcbf724a699c9db3f4e84bbe60192b3ecf679)) by @linrongbin16 ([#516](https://github.com/rsvim/rsvim/pull/516))

- *(consts)* Refactor path config home and entry file (#528) ([06e37627](https://github.com/rsvim/rsvim/commit/06e376272b38b690739b791419824f7654ead284)) by @linrongbin16 ([#528](https://github.com/rsvim/rsvim/pull/528))

- *(consts)* Replace constants with 'LazyLock' instead of 'OnceLock' (#529) ([aa5669aa](https://github.com/rsvim/rsvim/commit/aa5669aa764c29ff61916fcf890b16f72e335191)) by @linrongbin16 ([#529](https://github.com/rsvim/rsvim/pull/529))

- *(normal)* Refactor "append"/"newline" ops in normal mode (#535) ([e5eeb98a](https://github.com/rsvim/rsvim/commit/e5eeb98a4580ea7d4f526ceaeafee23967d98bf7)) by @linrongbin16 ([#535](https://github.com/rsvim/rsvim/pull/535))

- *(module)* Refactor js module to hide 'pub' fields in struct (#542) ([e6818214](https://github.com/rsvim/rsvim/commit/e68182141e4e21e09996f88047974f6b5bd31ab9)) by @linrongbin16 ([#542](https://github.com/rsvim/rsvim/pull/542))

- *(tree)* Avoid 'Rc' in tree relationships (#544) ([c6bfb06d](https://github.com/rsvim/rsvim/commit/c6bfb06dfbea2510222b62533546dd25e1e80cfe)) by @linrongbin16 ([#544](https://github.com/rsvim/rsvim/pull/544))

- *(pathcfg)* Mock 'path_config' for future testings (#545) ([40e96b4c](https://github.com/rsvim/rsvim/commit/40e96b4ccbdc9d6dacf8dce7762b5c93b63be4da)) by @linrongbin16 ([#545](https://github.com/rsvim/rsvim/pull/545))

- *(module)* Refactor js module fetching (#546) ([b1bd50c2](https://github.com/rsvim/rsvim/commit/b1bd50c2f0d5671e26fff60beaf6597eea4d8c51)) by @linrongbin16 ([#546](https://github.com/rsvim/rsvim/pull/546))

- *(js)* Add old "from scratch" constructor back to js runtime (#547) ([b5451c73](https://github.com/rsvim/rsvim/commit/b5451c734c4c824702817a17cd592bf15e060591)) by @linrongbin16 ([#547](https://github.com/rsvim/rsvim/pull/547))

- *(buf)* Drop unnecessary 'Rc' pointer inside text cache (#548) ([edd52f22](https://github.com/rsvim/rsvim/commit/edd52f22b39340ce2037921dbb05bfa7bc0d99ce)) by @linrongbin16 ([#548](https://github.com/rsvim/rsvim/pull/548))

- *(cli)* Use 'PathBuf' for cli arguments (#557) ([71c72e7f](https://github.com/rsvim/rsvim/commit/71c72e7fae9bd7a9295163e8b756a38c9fa6f5f0)) by @linrongbin16 ([#557](https://github.com/rsvim/rsvim/pull/557))

- *(cli)* Add '--headless' cli option for mocking event loop (#571) ([96fb6045](https://github.com/rsvim/rsvim/commit/96fb6045f0d677260289adcf58c55a8baa9dd18d)) by @linrongbin16 ([#571](https://github.com/rsvim/rsvim/pull/571))

- *(tui)* Move STDOUT out of "EventLoop" and create "StdoutWriter" (#573) ([573aaf97](https://github.com/rsvim/rsvim/commit/573aaf974690bc7ee79059a80d4775bceef40359)) by @linrongbin16 ([#573](https://github.com/rsvim/rsvim/pull/573))

- *(cli)* Refactor special cli options version and help (#574) ([7909ff58](https://github.com/rsvim/rsvim/commit/7909ff58d6624815cce3bc83b3d8f7a360e32613)) by @linrongbin16 ([#574](https://github.com/rsvim/rsvim/pull/574))

- *(ui)* Shorten window/command-line widget names (#589) ([63188e53](https://github.com/rsvim/rsvim/commit/63188e536d5487bbc307a92d15dea8dfb3da2388)) by @linrongbin16 ([#589](https://github.com/rsvim/rsvim/pull/589))

- *(state)* Move 'jsrt_tick_dispatcher' to data_access and refactor test cases (#592) ([50e32152](https://github.com/rsvim/rsvim/commit/50e32152a81bf97e9bbdf6e9a7d0d7165e706b4d)) by @linrongbin16 ([#592](https://github.com/rsvim/rsvim/pull/592))

- *(viewport)* Refactor mutable references on UI tree to avoid small copy/clone (#595) ([adea3c69](https://github.com/rsvim/rsvim/commit/adea3c6928698ac3cf7df312785feaa877ac1130)) by @linrongbin16 ([#595](https://github.com/rsvim/rsvim/pull/595))

- *(viewport)* Create new "EditableWidgetable" trait to drop duplicated matchings (#596) ([7993a859](https://github.com/rsvim/rsvim/commit/7993a859a35333f42f6f2142e751fefc59b36f89)) by @linrongbin16 ([#596](https://github.com/rsvim/rsvim/pull/596))

- *(evloop)* Refactor channels and messages (#599) ([04a15faf](https://github.com/rsvim/rsvim/commit/04a15faf879432a4d0f48eb6a709fe1466af5b6b)) by @linrongbin16 ([#599](https://github.com/rsvim/rsvim/pull/599))

- *(channel)* Refactor sync send messages "spawn_blocking" calls (#600) ([6b91ec70](https://github.com/rsvim/rsvim/commit/6b91ec701576ab4622d5c45e326eea4d20d0dfe8)) by @linrongbin16 ([#600](https://github.com/rsvim/rsvim/pull/600))

- *(cmd)* Move "ex commands" management and parsing logic to js runtime side (#601) ([db3e46ee](https://github.com/rsvim/rsvim/commit/db3e46ee604954c201be8eea3d03fde6d06cafcc)) by @linrongbin16 ([#601](https://github.com/rsvim/rsvim/pull/601))

- *(state)* Remove unused "state" structure (#606) ([847fa806](https://github.com/rsvim/rsvim/commit/847fa80637e5db60d7d86bac9abd3b51961cc85c)) by @linrongbin16 ([#606](https://github.com/rsvim/rsvim/pull/606))

- *(cli)* Add profile and git commit to version info (#611) ([a110259f](https://github.com/rsvim/rsvim/commit/a110259fc9110e88141787d250962b46b530d4bf)) by @linrongbin16 ([#611](https://github.com/rsvim/rsvim/pull/611))

- *(echo)* Directly update command-line widgets, not always sending message (#616) ([e90a9f01](https://github.com/rsvim/rsvim/commit/e90a9f01518fe7d1501e3934db1ea835499628e8)) by @linrongbin16 ([#616](https://github.com/rsvim/rsvim/pull/616))

- *(echo)* Still send echo message before editor/TUI is not initialized (#623) ([c3d505b0](https://github.com/rsvim/rsvim/commit/c3d505b0df429d91fbdcf95f4f9d2b2bdf661283)) by @linrongbin16 ([#623](https://github.com/rsvim/rsvim/pull/623))

- *(enum)* Refactor options with "strum" crate (#629) ([d3128f19](https://github.com/rsvim/rsvim/commit/d3128f193e10db93ba5e85cfbba9a166fb059bff)) by @linrongbin16 ([#629](https://github.com/rsvim/rsvim/pull/629))

- *(options)* Use "strum" for "FileEncodingOption" serialization (#630) ([1152b043](https://github.com/rsvim/rsvim/commit/1152b04321ac8cfaf758367e519e22e7fa3bfc88)) by @linrongbin16 ([#630](https://github.com/rsvim/rsvim/pull/630))

- *(js)* Simplify the async event loop of js runtime (#651) ([f355840a](https://github.com/rsvim/rsvim/commit/f355840a5e72a3a69dde94b5ddabfb9eaae6c3b0)) by @linrongbin16 ([#651](https://github.com/rsvim/rsvim/pull/651))

- *(config)* Moves global "PathConfig" variable into "EventLoop" (#668) ([ed076762](https://github.com/rsvim/rsvim/commit/ed07676208bfb85062fb7b4de8ca700d969c9395)) by @linrongbin16 ([#668](https://github.com/rsvim/rsvim/pull/668))

- *(ts)* Rewrite ts APIs with "interface" instead of "class" (#670) ([46158d1e](https://github.com/rsvim/rsvim/commit/46158d1ea14b39fe5dbf74ea837c009f7e4e1df0)) by @linrongbin16 ([#670](https://github.com/rsvim/rsvim/pull/670))

- *(string)* Avoid string clones (#674) ([a448333b](https://github.com/rsvim/rsvim/commit/a448333b2c843fa59e32688660c84160a064314b)) by @linrongbin16 ([#674](https://github.com/rsvim/rsvim/pull/674))

- *(cmd)* Refactor `Rsvim.cmd.list` and add `Rsvim.cmd.get` (#712) ([ccba7dd7](https://github.com/rsvim/rsvim/commit/ccba7dd7bf47796ae2956608ce4e102ca2f5a9a8)) by @linrongbin16 ([#712](https://github.com/rsvim/rsvim/pull/712))

- *(opt)* Remove duplicated "num_traits::clamp" calls (#715) ([0fcc568f](https://github.com/rsvim/rsvim/commit/0fcc568feee2c7b91d4da3be64b76ffaef2f9883)) by @linrongbin16 ([#715](https://github.com/rsvim/rsvim/pull/715))

- *(error)* Migrate all errors, anyhow to thiserror (#717) ([502d3e5d](https://github.com/rsvim/rsvim/commit/502d3e5d4aba93ba8db8489f1c53bc26763c6691)) by @linrongbin16 ([#717](https://github.com/rsvim/rsvim/pull/717))

- *(timer)* Migrate timer "delay" field from u64 to u32 (#721) ([5b08f45b](https://github.com/rsvim/rsvim/commit/5b08f45b2f0208e0904d04bea53c9185214a933f)) by @linrongbin16 ([#721](https://github.com/rsvim/rsvim/pull/721))

- *(bits)* Refactor multiple booleans with bitflags (#723) ([229a5857](https://github.com/rsvim/rsvim/commit/229a585740978c23a700815f9cefea72ea53cce1)) by @linrongbin16 ([#723](https://github.com/rsvim/rsvim/pull/723))

- *(opts)* Use paste! lower/upper/camel to reduce duplicated macro arguments (#727) ([1a3443a9](https://github.com/rsvim/rsvim/commit/1a3443a9803df33aea05414c8ab4f35cea7effcc)) by @linrongbin16 ([#727](https://github.com/rsvim/rsvim/pull/727))

- *(buf)* Refactor internal cache inside buffer (#732) ([d4bbfa16](https://github.com/rsvim/rsvim/commit/d4bbfa1608567759b5293705990c1e2aafc76170)) by @linrongbin16 ([#732](https://github.com/rsvim/rsvim/pull/732))

- *(init)* Refactor event loop initialization (#737) ([766fddf2](https://github.com/rsvim/rsvim/commit/766fddf29de9a8381a1a28fcb668278043ba586a)) by @linrongbin16 ([#737](https://github.com/rsvim/rsvim/pull/737))

- *(msg)* Migrate to unbounded channels for non blocking sending (#739) ([ed983260](https://github.com/rsvim/rsvim/commit/ed983260b0145c8a194522b46bc1020a7a3495e6)) by @linrongbin16 ([#739](https://github.com/rsvim/rsvim/pull/739))

- *(const)* Remove unused const (#740) ([a9a4242a](https://github.com/rsvim/rsvim/commit/a9a4242a40475577cd7c9b1f12e6c37a2c73211a)) by @linrongbin16 ([#740](https://github.com/rsvim/rsvim/pull/740))

- *(types)* Refactor typescript api types (#760) ([62412d53](https://github.com/rsvim/rsvim/commit/62412d53dcd5ac90ed14f5ae29ebb746d586af4e)) by @linrongbin16 ([#760](https://github.com/rsvim/rsvim/pull/760))

- *(js)* Add "async" keyword for async functions (#765) ([e246d3ab](https://github.com/rsvim/rsvim/commit/e246d3abae69f87c8dfcf6cb7b5c0c9bf44bfb58)) by @linrongbin16 ([#765](https://github.com/rsvim/rsvim/pull/765))

- *(js)* Keep typedoc comments for .d.ts declarations (#766) ([772acf3e](https://github.com/rsvim/rsvim/commit/772acf3e6cd79504a222c06ea1229f9aab19eaea)) by @linrongbin16 ([#766](https://github.com/rsvim/rsvim/pull/766))

- *(err)* Use "CompactString" for error message (#774) ([61d7a9bb](https://github.com/rsvim/rsvim/commit/61d7a9bb3f458fec3bd6762a37636427594ea7b7)) by @linrongbin16 ([#774](https://github.com/rsvim/rsvim/pull/774))

- *(ui)* Refactor logical shape and actual shape calculation (#778) ([e5419cac](https://github.com/rsvim/rsvim/commit/e5419cac40c10fb4ceb2585592d8ed0a7ae166ca)) by @linrongbin16 ([#778](https://github.com/rsvim/rsvim/pull/778))

- *(ui)* Refactor geo macros (#785) ([faa994fb](https://github.com/rsvim/rsvim/commit/faa994fbdf9308c95bcae66e12d29a79d3245caf)) by @linrongbin16 ([#785](https://github.com/rsvim/rsvim/pull/785))

- *(ui)* Refactor geo types and constructors (#796) ([cb9aeb66](https://github.com/rsvim/rsvim/commit/cb9aeb6690c8a501b4607cb93858abaf34b9fcbb)) by @linrongbin16 ([#796](https://github.com/rsvim/rsvim/pull/796))

- *(ui)* Refactor geo coordinates (#804) ([bf2aeebc](https://github.com/rsvim/rsvim/commit/bf2aeebc769d50d2674e8833ab3f9e1d6282d117)) by @linrongbin16 ([#804](https://github.com/rsvim/rsvim/pull/804))

- *(ui)* Remove unused "depth" property on node (#805) ([1edd086f](https://github.com/rsvim/rsvim/commit/1edd086ff5de221ba54532978e46f45fe5aa0390)) by @linrongbin16 ([#805](https://github.com/rsvim/rsvim/pull/805))

- *(test)* Refactor common utility functions in unit test (#808) ([2185566c](https://github.com/rsvim/rsvim/commit/2185566c2bc5a12a7bef8ef97e5bd5a9cc48fb72)) by @linrongbin16 ([#808](https://github.com/rsvim/rsvim/pull/808))

- *(test)* Fix window shape constructor in unit test (#814) ([3c2d14e3](https://github.com/rsvim/rsvim/commit/3c2d14e3f8d01060a64eb4107145ec1784c313c7)) by @linrongbin16 ([#814](https://github.com/rsvim/rsvim/pull/814))

- *(ui)* Refactor relative shape to absolute shape conversion (#815) ([a51b3321](https://github.com/rsvim/rsvim/commit/a51b3321cf8b3be51fb105b06c30328132985f47)) by @linrongbin16 ([#815](https://github.com/rsvim/rsvim/pull/815))

- *(ui)* Rename "RootContainer" to "Panel" and drop duplicates (#818) ([9b6cef8e](https://github.com/rsvim/rsvim/commit/9b6cef8e16b1b35d48bca635e2ba2f60d2ff1cc1)) by @linrongbin16 ([#818](https://github.com/rsvim/rsvim/pull/818))

- *(ui)* Removes unused "z-index" attribute in tree node (#820) ([01fd9c0d](https://github.com/rsvim/rsvim/commit/01fd9c0da7ba6ef1ee5434f2f1a0f98f94846d71)) by @linrongbin16 ([#820](https://github.com/rsvim/rsvim/pull/820))

- *(cmdline)* Add cmdline clear input method (#825) ([02d4907d](https://github.com/rsvim/rsvim/commit/02d4907d8df5ab3b9e63a323deee62aec4eee5b2)) by @linrongbin16 ([#825](https://github.com/rsvim/rsvim/pull/825))

- *(ui)* Use "U16Size" instead of "U16Rect" for more precise parameter (#826) ([94b2beb7](https://github.com/rsvim/rsvim/commit/94b2beb7a70350f293c3ab70a46b3b9f370ebd36)) by @linrongbin16 ([#826](https://github.com/rsvim/rsvim/pull/826))

- *(ui)* Remove "root_node" parameter from tree constructor method (#829) ([a9041fb7](https://github.com/rsvim/rsvim/commit/a9041fb751c4de35219699d650882b8326bb4577)) by @linrongbin16 ([#829](https://github.com/rsvim/rsvim/pull/829))

- *(ui)* Introduce taffy as layout engine (#830) ([00074292](https://github.com/rsvim/rsvim/commit/000742920af5fe9b8762d7865f76d4bbf1b1279d)) by @linrongbin16 ([#830](https://github.com/rsvim/rsvim/pull/830))

- *(lazy)* Replace std "LazyLock" with once_cell "Lazy" with parking_lot (#855) ([717a2d45](https://github.com/rsvim/rsvim/commit/717a2d459c8645e275233a618bb7094fff76fe58)) by @linrongbin16 ([#855](https://github.com/rsvim/rsvim/pull/855))

- *(id)* Refactor integer id with struct for type safe (#856) ([308e3934](https://github.com/rsvim/rsvim/commit/308e3934c7f5f8877df1771157c8005fac819a91)) by @linrongbin16 ([#856](https://github.com/rsvim/rsvim/pull/856))

- *(ringbuf)* Refactor single and double ended ring buffer (#871) ([17d2bbe2](https://github.com/rsvim/rsvim/commit/17d2bbe2220c722f2068fc262bb19a9cf78ce1d7)) by @linrongbin16 ([#871](https://github.com/rsvim/rsvim/pull/871))

- *(text)* Refactor text editing api (#874) ([06efd29a](https://github.com/rsvim/rsvim/commit/06efd29a70fdb2504a90a812c548f5df916ca40c)) by @linrongbin16 ([#874](https://github.com/rsvim/rsvim/pull/874))

- *(deps)* Remove unused "swc_sourcemap" crate (#875) ([1c64b90c](https://github.com/rsvim/rsvim/commit/1c64b90c52dea2625a1bc7b72d850d8ae6cc6cbe)) by @linrongbin16 ([#875](https://github.com/rsvim/rsvim/pull/875))

- *(id)* Go back to initial if hit max value (#876) ([9baaada2](https://github.com/rsvim/rsvim/commit/9baaada2f4909287c0c93fcf337315277d2949b2)) by @linrongbin16 ([#876](https://github.com/rsvim/rsvim/pull/876))

- *(text)* Refactor remove text api (#877) ([76f280ff](https://github.com/rsvim/rsvim/commit/76f280ff8ba7cfbfcdea54b2cf6bef0d8b5da44c)) by @linrongbin16 ([#877](https://github.com/rsvim/rsvim/pull/877))

- *(syn)* Add editings to syntax pending job queue and send "SyntaxEditReq" message (#884) ([c182f7b5](https://github.com/rsvim/rsvim/commit/c182f7b5c1ab0b991474fab9517301afb02bc925)) by @linrongbin16 ([#884](https://github.com/rsvim/rsvim/pull/884))

- *(buf)* Replace "Rc<String>" with "arcstr::ArcStr" (#887) ([7368272b](https://github.com/rsvim/rsvim/commit/7368272b08adfaad2c05e793737e9ef6e950f931)) by @linrongbin16 ([#887](https://github.com/rsvim/rsvim/pull/887))

- *(buf)* Refactor text removable chars range api (#896) ([45388a16](https://github.com/rsvim/rsvim/commit/45388a16e1b06d5db37a91089e0a73787868b408)) by @linrongbin16 ([#896](https://github.com/rsvim/rsvim/pull/896))

- *(undo)* Add cursor char position before and after editing (#899) ([db6cafc9](https://github.com/rsvim/rsvim/commit/db6cafc9ce40d70e401340772220518642baa02e)) by @linrongbin16 ([#899](https://github.com/rsvim/rsvim/pull/899))

- *(undo)* Save cursor char position to undo manager (#900) ([0397ff8a](https://github.com/rsvim/rsvim/commit/0397ff8aaacfc6f3d433cd21c884ab7f2bac6ff5)) by @linrongbin16 ([#900](https://github.com/rsvim/rsvim/pull/900))

- *(hl)* Add treesitter Query (#930) ([c6d2f0d1](https://github.com/rsvim/rsvim/commit/c6d2f0d1e187461773cf1b5681f9f37563a4cf8f)) by @linrongbin16 ([#930](https://github.com/rsvim/rsvim/pull/930))

- *(syn)* Capture highlight names (#955) ([4494c27a](https://github.com/rsvim/rsvim/commit/4494c27ae7481452e0b8be87d724b47f3fc87844)) by @linrongbin16 ([#955](https://github.com/rsvim/rsvim/pull/955))

- *(hl)* Add default color name (#956) ([b7f78c46](https://github.com/rsvim/rsvim/commit/b7f78c4601ecc6844a1aeafff960bbcee1d3bd09)) by @linrongbin16 ([#956](https://github.com/rsvim/rsvim/pull/956))

- *(hl)* Refactor captured query byte indexing to char indexing for better response (#957) ([f96e94de](https://github.com/rsvim/rsvim/commit/f96e94de04bfd8a634f48b8755e746e35123d44e)) by @linrongbin16 ([#957](https://github.com/rsvim/rsvim/pull/957))

- *(hl)* Refactor highlight resolve (#958) ([622f3337](https://github.com/rsvim/rsvim/commit/622f333760411ab715b65fd803319e084d638285)) by @linrongbin16 ([#958](https://github.com/rsvim/rsvim/pull/958))

- *(ui)* Fill dirty_rows in canvas without for-loop (#973) ([fb57233c](https://github.com/rsvim/rsvim/commit/fb57233cffdd93f73241e188b403948b6d5e89f3)) by @linrongbin16 ([#973](https://github.com/rsvim/rsvim/pull/973))

- *(color)* Rename "ui.foreground" to "ui.text" (#976) ([702aa919](https://github.com/rsvim/rsvim/commit/702aa919642501c228912adeef46ebff4f161d12)) by @linrongbin16 ([#976](https://github.com/rsvim/rsvim/pull/976))

- *(buf)* Make end-of-line char detection api public  (#981) ([8f30fed4](https://github.com/rsvim/rsvim/commit/8f30fed44b9eb5ab3db293195138044cc9db2e26)) by @linrongbin16 ([#981](https://github.com/rsvim/rsvim/pull/981))

- *(cursor)* Remove useless legacy logic when scrolling window/viewport (#988) ([1c4a828e](https://github.com/rsvim/rsvim/commit/1c4a828ed24675bb3ceac9a1baae79ab60362c5a)) by @linrongbin16 ([#988](https://github.com/rsvim/rsvim/pull/988))

- *(ui)* Refactor viewport line processing logic when wrap=false (#989) ([81a95589](https://github.com/rsvim/rsvim/commit/81a955891ab3c9d7c906d20fb20f49a4e7f94fad)) by @linrongbin16 ([#989](https://github.com/rsvim/rsvim/pull/989))

- *(ui)* Refactor line processing logic when wrap=true & linebreak=false (#990) ([3891fc9d](https://github.com/rsvim/rsvim/commit/3891fc9d02855e7f06cfe44b5414b46ef6d8dcd2)) by @linrongbin16 ([#990](https://github.com/rsvim/rsvim/pull/990))

- *(ui)* Use "Arc" on colorscheme to avoid structure clone (#1008) ([1229fc00](https://github.com/rsvim/rsvim/commit/1229fc00e858f525ae8a470685647875f36597e9)) by @linrongbin16 ([#1008](https://github.com/rsvim/rsvim/pull/1008))

- *(state)* Rename "StateDataAccess" to "StateContext" (#1009) ([5f3f8c76](https://github.com/rsvim/rsvim/commit/5f3f8c7683fb5ee5ca3307c3fe4cd687ead822e9)) by @linrongbin16 ([#1009](https://github.com/rsvim/rsvim/pull/1009))

- *(cli)* Restore clap cli options (#1018) ([6a4fc24f](https://github.com/rsvim/rsvim/commit/6a4fc24ff74f12a745a3f9944f6c5276b1dbf2fb)) by @linrongbin16 ([#1018](https://github.com/rsvim/rsvim/pull/1018))

- *(color)* Split "syn"/"color" module, drop "parking_lot" from tokio (#1019) ([e0e4e4d0](https://github.com/rsvim/rsvim/commit/e0e4e4d015ebf9369203b1cacaa71ab1fcc0fb83)) by @linrongbin16 ([#1019](https://github.com/rsvim/rsvim/pull/1019))

- *(ui)* Stop hide and recover cursor if text is not changed (#1025) ([025857d5](https://github.com/rsvim/rsvim/commit/025857d5a57f13cd809185c70b842da8c1b619b4)) by @linrongbin16 ([#1025](https://github.com/rsvim/rsvim/pull/1025))

- *(syn)* Rename "language" to "grammar" (#1020) ([858312a9](https://github.com/rsvim/rsvim/commit/858312a95a2921bbf2ee8605659892956b31bb39)) by @linrongbin16 ([#1020](https://github.com/rsvim/rsvim/pull/1020))

- *(syn)* Use "parser-lib-path" option when building tree-sitter parsers (#1043) ([16db29dd](https://github.com/rsvim/rsvim/commit/16db29dddb881df331a94531493e1580287025a2)) by @linrongbin16 ([#1043](https://github.com/rsvim/rsvim/pull/1043))

- *(syn)* Refactor tree-sitter loader "parser-lib-path" (#1044) ([13ccd408](https://github.com/rsvim/rsvim/commit/13ccd408f9b042f318fa1a7b7604a7e1f0a48c84)) by @linrongbin16 ([#1044](https://github.com/rsvim/rsvim/pull/1044))

- *(util)* Refactor file path extension utils (#1053) ([01f47187](https://github.com/rsvim/rsvim/commit/01f4718700bff04ccac698c068533e9eddd39e94)) by @linrongbin16 ([#1053](https://github.com/rsvim/rsvim/pull/1053))

- *(syn)* Associate tree-sitter file types with parser (#1054) ([262cf531](https://github.com/rsvim/rsvim/commit/262cf531d315cac27a642eced849cbd9d37c368b)) by @linrongbin16 ([#1054](https://github.com/rsvim/rsvim/pull/1054))

- *(syn)* Rename "grammar" word to "parser" for user api (#1055) ([534df50b](https://github.com/rsvim/rsvim/commit/534df50b89f43528c0230f55dc53165fe29c7471)) by @linrongbin16 ([#1055](https://github.com/rsvim/rsvim/pull/1055))

- *(resource)* Add resource table to manage file/child process/sockets (#1078) ([becb1bd8](https://github.com/rsvim/rsvim/commit/becb1bd8a28285b09419fa1445fe3692d560b866)) by @linrongbin16 ([#1078](https://github.com/rsvim/rsvim/pull/1078))

- *(js)* Add "ToV8" macro to reduce duplicated code logic (#1086) ([53e6757b](https://github.com/rsvim/rsvim/commit/53e6757bb5cdea118cae99e0a8f5d0ada1c07620)) by @linrongbin16 ([#1086](https://github.com/rsvim/rsvim/pull/1086))


### <!-- 5 -->Testing

- *(insert)* New test for ascii characters (writable and extended) (#532) ([5a9d63f8](https://github.com/rsvim/rsvim/commit/5a9d63f82e7a0978d73fe4c857ba21101702047e)) by @jackcat13 ([#532](https://github.com/rsvim/rsvim/pull/532))

- *(loader)* Add tests for js fs module loader (#531) ([8b895c8e](https://github.com/rsvim/rsvim/commit/8b895c8e2a175ccdf8b3da09617060470cb1315e)) by @linrongbin16 ([#531](https://github.com/rsvim/rsvim/pull/531))

- *(unicode)* Fix test cases for wide unicode chars canvas drawing (#541) ([f1b55485](https://github.com/rsvim/rsvim/commit/f1b554853fbd1985e9ef8616e2890c622a4d1abd)) by @linrongbin16 ([#541](https://github.com/rsvim/rsvim/pull/541))

- *(event)* Add mock event stream and test 'setTimeout' web api (#576) ([c66fb581](https://github.com/rsvim/rsvim/commit/c66fb5813bc8df38672e01bcfa56c4a142645cec)) by @linrongbin16 ([#576](https://github.com/rsvim/rsvim/pull/576))

- *(evloop)* Add "run_with_mock_operations" for operation based loop run (#612) ([ded68501](https://github.com/rsvim/rsvim/commit/ded68501b5afec29db86d4117ef32ea232927da7)) by @linrongbin16 ([#612](https://github.com/rsvim/rsvim/pull/612))

- *(js)* Test empty arguments for import.meta.resolve (#666) ([d75dbdbc](https://github.com/rsvim/rsvim/commit/d75dbdbc2f2a301c71f909ee9394b8b3669132ee)) by @linrongbin16 ([#666](https://github.com/rsvim/rsvim/pull/666))

- *(js)* Fix module map tests (#669) ([567b2a29](https://github.com/rsvim/rsvim/commit/567b2a29726c4b99d203f723fc9b4613bc662ddc)) by @linrongbin16 ([#669](https://github.com/rsvim/rsvim/pull/669))

- *(module)* Fix lazy initialized "FsModuleLoader" in testing (#685) ([a928730c](https://github.com/rsvim/rsvim/commit/a928730c83453e1884ebbd2060500ccf2cf4ea5f)) by @linrongbin16 ([#685](https://github.com/rsvim/rsvim/pull/685))

- *(js)* Add 1 test case for invalid js scripts error (#695) ([214a2ae9](https://github.com/rsvim/rsvim/commit/214a2ae9c52f121e3cd7575709c84a0994a76d04)) by @linrongbin16 ([#695](https://github.com/rsvim/rsvim/pull/695))

- *(undo)* Test undo with buffer editing (#898) ([b0fb17d7](https://github.com/rsvim/rsvim/commit/b0fb17d72f557998dd46e34c79715019a0804b79)) by @linrongbin16 ([#898](https://github.com/rsvim/rsvim/pull/898))

- *(syn)* Test syntax parsing when buffer editing (#901) ([d527e52c](https://github.com/rsvim/rsvim/commit/d527e52c7095977d3f23ce9d524da61cedb69250)) by @linrongbin16 ([#901](https://github.com/rsvim/rsvim/pull/901))

- *(syn)* Add more tests for syntax parsing when editing buffers (#902) ([ece51d00](https://github.com/rsvim/rsvim/commit/ece51d00b2a305bc9abefe1dfce21878c3f66f97)) by @linrongbin16 ([#902](https://github.com/rsvim/rsvim/pull/902))

- *(hl)* Add more failure hl tests (#934) ([6398ac22](https://github.com/rsvim/rsvim/commit/6398ac222e4ba4d2af610abe047f32fd57a7f72e)) by @linrongbin16 ([#934](https://github.com/rsvim/rsvim/pull/934))

- *(hl)* Print canvas highlights (#965) ([2365ab09](https://github.com/rsvim/rsvim/commit/2365ab0984b26e385482909f3511d86e800415b9)) by @linrongbin16 ([#965](https://github.com/rsvim/rsvim/pull/965))

- *(ui)* Use random input to test "Viewport::search" (#1012) ([e1ac962f](https://github.com/rsvim/rsvim/commit/e1ac962f0c2ba075d34c9ab1f0777dfa5efc381c)) by @linrongbin16 ([#1012](https://github.com/rsvim/rsvim/pull/1012))

- *(syn)* Test syntax grammar loader (#1032) ([37a7432d](https://github.com/rsvim/rsvim/commit/37a7432d565eff99ba0d174b35d5eb578a178dad)) by @linrongbin16 ([#1032](https://github.com/rsvim/rsvim/pull/1032))

