## [0.1.2-alpha.1](https://github.com/rsvim/rsvim/compare/v0.1.1..0.1.2-alpha.1) - 2025-08-28

[6b2e14ea](https://github.com/rsvim/rsvim/commit/6b2e14ea3e91b9074116a796e0119f4427efb7b7)...[0edc36c7](https://github.com/rsvim/rsvim/commit/0edc36c74fa305d0d034aadb8147e9915be3fda1)

### <!-- 1 -->Bug Fixes

- *(build)* Fix git repo not found when install with "cargo install" (#627) ([0edc36c7](https://github.com/rsvim/rsvim/commit/0edc36c74fa305d0d034aadb8147e9915be3fda1)) by @linrongbin16 ([#627](https://github.com/rsvim/rsvim/pull/627))

## [0.1.1-beta.1](https://github.com/rsvim/rsvim/compare/v0.1.1-alpha.10..v0.1.1-beta.1) - 2025-08-27

[3f21c820](https://github.com/rsvim/rsvim/commit/3f21c820de70cae32720139252c8f198f398ad45)...[31ea70b5](https://github.com/rsvim/rsvim/commit/31ea70b58d406cfaacf277f5cf4ae6f5fea6b502)

### <!-- 0 -->Features

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


### <!-- 1 -->Bug Fixes

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


### <!-- 2 -->Performance Improvements

- *(canvas)* Use "clone_from" when cloning frame (#511) ([61c7500b](https://github.com/rsvim/rsvim/commit/61c7500b48dfd08f07d2b11356b68a1a6d3bb2a9)) by @linrongbin16 ([#511](https://github.com/rsvim/rsvim/pull/511))

- *(cli)* Move to "fern" to reduce binary size (#567) ([fde0bb33](https://github.com/rsvim/rsvim/commit/fde0bb3384f1e706403126e024c2ffdd1a61ecd1)) by @linrongbin16 ([#567](https://github.com/rsvim/rsvim/pull/567))

- *(cli)* Move to "lexopt" to reduce binary size (#568) ([677005dc](https://github.com/rsvim/rsvim/commit/677005dc5f4b08bacc4c447ea592ffa8100f2be3)) by @linrongbin16 ([#568](https://github.com/rsvim/rsvim/pull/568))


### <!-- 3 -->Code Refactoring

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


### <!-- 5 -->Testing

- *(insert)* New test for ascii characters (writable and extended) (#532) ([5a9d63f8](https://github.com/rsvim/rsvim/commit/5a9d63f82e7a0978d73fe4c857ba21101702047e)) by @jackcat13 ([#532](https://github.com/rsvim/rsvim/pull/532))

- *(loader)* Add tests for js fs module loader (#531) ([8b895c8e](https://github.com/rsvim/rsvim/commit/8b895c8e2a175ccdf8b3da09617060470cb1315e)) by @linrongbin16 ([#531](https://github.com/rsvim/rsvim/pull/531))

- *(unicode)* Fix test cases for wide unicode chars canvas drawing (#541) ([f1b55485](https://github.com/rsvim/rsvim/commit/f1b554853fbd1985e9ef8616e2890c622a4d1abd)) by @linrongbin16 ([#541](https://github.com/rsvim/rsvim/pull/541))

- *(event)* Add mock event stream and test 'setTimeout' web api (#576) ([c66fb581](https://github.com/rsvim/rsvim/commit/c66fb5813bc8df38672e01bcfa56c4a142645cec)) by @linrongbin16 ([#576](https://github.com/rsvim/rsvim/pull/576))

- *(evloop)* Add "run_with_mock_operations" for operation based loop run (#612) ([ded68501](https://github.com/rsvim/rsvim/commit/ded68501b5afec29db86d4117ef32ea232927da7)) by @linrongbin16 ([#612](https://github.com/rsvim/rsvim/pull/612))

## [0.1.1-alpha.10](https://github.com/rsvim/rsvim/compare/v0.1.1-alpha.9..v0.1.1-alpha.10) - 2025-06-09

[d8577472](https://github.com/rsvim/rsvim/commit/d8577472c1180a5a4faa2b16a58182f3369b9dd3)...[3f21c820](https://github.com/rsvim/rsvim/commit/3f21c820de70cae32720139252c8f198f398ad45)

### <!-- 0 -->Features

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


### <!-- 1 -->Bug Fixes

- *(tui)* Recover terminal and save backtrace info when panic (#395) ([2299f0ef](https://github.com/rsvim/rsvim/commit/2299f0efdce62cc0013e5ae8bbc523d3698f9a6e)) by @linrongbin16 ([#395](https://github.com/rsvim/rsvim/pull/395))

- *(viewport)* Refactor wrap line anchor searching algorithms and fix edge cases (#407) ([f9c50ed6](https://github.com/rsvim/rsvim/commit/f9c50ed623d9bb97cb712b20e7f945e0e80f0387)) by @linrongbin16 ([#407](https://github.com/rsvim/rsvim/pull/407))

- *(cursor)* Limit cursor position by last visible char when back to normal mode (#438) ([b1225e8e](https://github.com/rsvim/rsvim/commit/b1225e8efd50ec41e0e31d4bbff4790dee1fd81c)) by @linrongbin16 ([#438](https://github.com/rsvim/rsvim/pull/438))

- *(cursor)* Fix cursor motion legacy boundary to work along anchor search (#440) ([a938add6](https://github.com/rsvim/rsvim/commit/a938add61daf2b0b58dacb64ed22e36a353db3cd)) by @linrongbin16 ([#440](https://github.com/rsvim/rsvim/pull/440))

- *(build)* Fix debug assertions warnings for release build (#446) ([68829ab2](https://github.com/rsvim/rsvim/commit/68829ab2610fc1e3d69d5f724d7178f1c10ed0f8)) by @linrongbin16 ([#446](https://github.com/rsvim/rsvim/pull/446))

- *(insert)* Fix non 1-width unicode chars insert (#452) ([2e2adb42](https://github.com/rsvim/rsvim/commit/2e2adb429177c09168ec409ef6d1f6020d164dda)) by @linrongbin16 ([#452](https://github.com/rsvim/rsvim/pull/452))


### <!-- 2 -->Performance Improvements

- *(cursor)* Reduce 1 lock/unlock call during cursor movement (#383) ([ce0dbfd8](https://github.com/rsvim/rsvim/commit/ce0dbfd8d20bcb405682c80828dd1be6e9f1bba9)) by @linrongbin16 ([#383](https://github.com/rsvim/rsvim/pull/383))

- *(cursor)* Drop duplicated normalization between window scroll "to" and "by" (#436) ([7543c731](https://github.com/rsvim/rsvim/commit/7543c73134b9ed840b5b883963aa99b2dd76be4a)) by @linrongbin16 ([#436](https://github.com/rsvim/rsvim/pull/436))

- *(viewport)* More compact memory layout (#441) ([4bf887fb](https://github.com/rsvim/rsvim/commit/4bf887fba055b49743a9230ed7ddeeb5c2f34ad1)) by @linrongbin16 ([#441](https://github.com/rsvim/rsvim/pull/441))

- *(viewport)* Drop unnecessary locks on readyonly viewport (#443) ([b6b91832](https://github.com/rsvim/rsvim/commit/b6b91832d900775c538fdda905d4724e3e51de31)) by @linrongbin16 ([#443](https://github.com/rsvim/rsvim/pull/443))


### <!-- 3 -->Code Refactoring

- *(cursor)* Extract duplicated code logic into single method (#384) ([8be594e5](https://github.com/rsvim/rsvim/commit/8be594e531fc69e0dfaf1740ba464a2c3dfbd85c)) by @linrongbin16 ([#384](https://github.com/rsvim/rsvim/pull/384))

- *(viewport)* Reduce duplicated logic with line-wise func arg (#397) ([18badf90](https://github.com/rsvim/rsvim/commit/18badf908d2101ace70e97ae7717c1419d922342)) by @linrongbin16 ([#397](https://github.com/rsvim/rsvim/pull/397))

- *(viewport)* Refactor internal leftward/rightward searching api (#399) ([0ce7f9e3](https://github.com/rsvim/rsvim/commit/0ce7f9e3dbe2628e8792527cccc06be7d159b1e1)) by @linrongbin16 ([#399](https://github.com/rsvim/rsvim/pull/399))

- *(buf)* Refactor buffer's last char apis (#403) ([57fc324a](https://github.com/rsvim/rsvim/commit/57fc324a34553a472e79b1f952a756546c7a1dbe)) by @linrongbin16 ([#403](https://github.com/rsvim/rsvim/pull/403))

- *(buf)* Rename last char on line api (#405) ([77d74901](https://github.com/rsvim/rsvim/commit/77d749012babbc9545f4e69cc50b22d5e768295a)) by @linrongbin16 ([#405](https://github.com/rsvim/rsvim/pull/405))

- *(viewport)* Simplify the "cannot fully contain target line" logic (#414) ([49554168](https://github.com/rsvim/rsvim/commit/495541680d724c171f00611511894842d6ce2d03)) by @linrongbin16 ([#414](https://github.com/rsvim/rsvim/pull/414))

- *(cursor)* Refactor cursor motion in normal mode (#433) ([f3a6ef50](https://github.com/rsvim/rsvim/commit/f3a6ef50f272075558f965f3e4098f346e1fb673)) by @linrongbin16 ([#433](https://github.com/rsvim/rsvim/pull/433))

- *(state)* Add "handle_op" api for low-level operation layer (#447) ([429f8cc4](https://github.com/rsvim/rsvim/commit/429f8cc43f676cce1a548ac085651d95d7b1133d)) by @linrongbin16 ([#447](https://github.com/rsvim/rsvim/pull/447))

## [0.1.1-alpha.9](https://github.com/rsvim/rsvim/compare/v0.1.1-alpha.8..v0.1.1-alpha.9) - 2025-05-20

[feafb680](https://github.com/rsvim/rsvim/commit/feafb680893ffdece400e1b17b7cb02b18a3ffc8)...[d8577472](https://github.com/rsvim/rsvim/commit/d8577472c1180a5a4faa2b16a58182f3369b9dd3)

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


### <!-- 1 -->Bug Fixes

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


### <!-- 2 -->Performance Improvements

- *(start)* Move built-in runtime modules evaluation to snapshot phase (#211) ([8003a3e4](https://github.com/rsvim/rsvim/commit/8003a3e47153ea1e6fc208c8934eb256c2fe119f)) by @linrongbin16 ([#211](https://github.com/rsvim/rsvim/pull/211))

- *(hash)* Use ahash instead of std lib (#236) ([c5d58b66](https://github.com/rsvim/rsvim/commit/c5d58b6679fb09cfbeae9ba4dd9e12ad352ab307)) by @linrongbin16 ([#236](https://github.com/rsvim/rsvim/pull/236))

- *(buf)* Cache line-wise chars index and display width (#303) ([b97c52e1](https://github.com/rsvim/rsvim/commit/b97c52e18b28fbb71e23a73da6d098fbc3a69075)) by @linrongbin16 ([#303](https://github.com/rsvim/rsvim/pull/303))

- *(cursor)* Reduce lock in moving cursor and scrolling window motion (#352) ([3935cc8b](https://github.com/rsvim/rsvim/commit/3935cc8b145ea26b4f6a6115b7d43288cc16cf6a)) by @linrongbin16 ([#352](https://github.com/rsvim/rsvim/pull/352))

- *(size)* Move version string to build.rs to reduce binary size (#366) ([048daef0](https://github.com/rsvim/rsvim/commit/048daef050552c6ba8dbc70403d31c126cb5c943)) by @linrongbin16 ([#366](https://github.com/rsvim/rsvim/pull/366))


### <!-- 3 -->Code Refactoring

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

- *(js)* Add 'useDefineForClassFields' for tsconfig (#332) ([72646836](https://github.com/rsvim/rsvim/commit/72646836d79a51617ab100149d343ca3b452dc74)) by @linrongbin16 ([#332](https://github.com/rsvim/rsvim/pull/332))

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

## [0.1.1-alpha.8](https://github.com/rsvim/rsvim/compare/v0.1.1-alpha.5..v0.1.1-alpha.8) - 2024-10-19

[6e6c03c7](https://github.com/rsvim/rsvim/commit/6e6c03c7f828a5acbfae029370d061b5e907dd1e)...[feafb680](https://github.com/rsvim/rsvim/commit/feafb680893ffdece400e1b17b7cb02b18a3ffc8)

### <!-- 0 -->Features

- *(script)* Read config file (#116) ([2fc10292](https://github.com/rsvim/rsvim/commit/2fc10292f6416ea76b47e8115776caedcbbeded0)) by @linrongbin16 ([#116](https://github.com/rsvim/rsvim/pull/116))

- *(script)* Start js runtime in a separate thread (#116) ([2fc10292](https://github.com/rsvim/rsvim/commit/2fc10292f6416ea76b47e8115776caedcbbeded0)) by @linrongbin16 ([#116](https://github.com/rsvim/rsvim/pull/116))

- *(script)* Implement js runtime with deno_core (#126) ([43d02884](https://github.com/rsvim/rsvim/commit/43d028844d5ea96ca240ebae6c728c316a9d2553)) by @linrongbin16 ([#126](https://github.com/rsvim/rsvim/pull/126))

- *(defaults)* Add config file and cache dir path (#127) ([f199ae47](https://github.com/rsvim/rsvim/commit/f199ae47a9f641fe8e2fa4c2f0840fe90c9cecdf)) by @linrongbin16 ([#127](https://github.com/rsvim/rsvim/pull/127))

- *(script)* Use v8 along with tokio runtime (#131) ([0e4189aa](https://github.com/rsvim/rsvim/commit/0e4189aa2c06464f84be282de6e61e0991a4251e)) by @linrongbin16 ([#131](https://github.com/rsvim/rsvim/pull/131))

- *(js)* Expose __InternalVimGlobalObject (#136) ([c5a4356c](https://github.com/rsvim/rsvim/commit/c5a4356c58fd707b0c0280b17d991d1d686a6285)) by @linrongbin16 ([#136](https://github.com/rsvim/rsvim/pull/136))

- *(js)* Add line wrap APIs (#138) ([1ff094a6](https://github.com/rsvim/rsvim/commit/1ff094a67086cca596939601eac5018c4ea537f6)) by @linrongbin16 ([#138](https://github.com/rsvim/rsvim/pull/138))

- *(js)* Add "setTimeout" API (#151) ([f3344480](https://github.com/rsvim/rsvim/commit/f33444804376221a3ec58925bb6bce5af12f9b9f)) by @linrongbin16 ([#151](https://github.com/rsvim/rsvim/pull/151))

- *(opt)* Add "line-break" option, rename "line-wrap" to "wrap" (#166) ([71064d55](https://github.com/rsvim/rsvim/commit/71064d559cf34fd8e4f1a742d088269caba3c631)) by @linrongbin16 ([#166](https://github.com/rsvim/rsvim/pull/166))

- *(js)* Add "breakAt" opt (#167) ([beb5c5d2](https://github.com/rsvim/rsvim/commit/beb5c5d28c23d4d663d08d727997e97c66403525)) by @linrongbin16 ([#167](https://github.com/rsvim/rsvim/pull/167))

- *(cli)* Add "rusty_v8" and "swc" version (#168) ([322ac341](https://github.com/rsvim/rsvim/commit/322ac3413fc3484d84fa0a8f6f7cc7821abaea37)) by @linrongbin16 ([#168](https://github.com/rsvim/rsvim/pull/168))

- *(opt)* Render wrap (#171) ([298d8f86](https://github.com/rsvim/rsvim/commit/298d8f860402f8a73c85d3570df0849eb6244752)) by @linrongbin16 ([#171](https://github.com/rsvim/rsvim/pull/171))

- *(ui)* Render "line-break" option (#182) ([809b983c](https://github.com/rsvim/rsvim/commit/809b983c9193303d0d05674ccef5553ce53b3326)) by @linrongbin16 ([#182](https://github.com/rsvim/rsvim/pull/182))


### <!-- 1 -->Bug Fixes

- *(canvas)* Fix dirty detection and tests, update help (#119) ([986f8b33](https://github.com/rsvim/rsvim/commit/986f8b33b81df0e375d0a2dc16ef97f9e8c6373a)) by @linrongbin16 ([#119](https://github.com/rsvim/rsvim/pull/119))

- *(shutdown)* Detached and blocked tracker (#169) ([2b45cf74](https://github.com/rsvim/rsvim/commit/2b45cf740863b2994d6f0cdb4a0fa27bded8316e)) by @linrongbin16 ([#169](https://github.com/rsvim/rsvim/pull/169))

- *(ui)* Fix missing whitespaces when "line-break" on (#197) ([0d1410e2](https://github.com/rsvim/rsvim/commit/0d1410e230780e64def866dc5968cd22c17cf57c)) by @linrongbin16 ([#197](https://github.com/rsvim/rsvim/pull/197))


### <!-- 2 -->Performance Improvements

- *(start)* Make snapshot v3 (#199) ([2f04e781](https://github.com/rsvim/rsvim/commit/2f04e7810b300937e1276f567764426a1c5eb69b)) by @linrongbin16 ([#199](https://github.com/rsvim/rsvim/pull/199))

- *(start)* Initialize built-in modules with snapshot (#205) ([73a92c0c](https://github.com/rsvim/rsvim/commit/73a92c0c3d20ac7c66a184987ce4024ad4918c7c)) by @linrongbin16 ([#205](https://github.com/rsvim/rsvim/pull/205))

- *(start)* Compress snapshot blob (#206) ([ac8420d1](https://github.com/rsvim/rsvim/commit/ac8420d17605aef8f975fdb5000774017a9f0384)) by @linrongbin16 ([#206](https://github.com/rsvim/rsvim/pull/206))


### <!-- 3 -->Code Refactoring

- *(main)* Explicitly create tokio runtime instead of the `main` macro (#120) ([ef87ffd4](https://github.com/rsvim/rsvim/commit/ef87ffd4dd7669e297c0d7773050245760f86f7c)) by @linrongbin16 ([#120](https://github.com/rsvim/rsvim/pull/120))

- *(shutdown)* Graceful shutdown (#125) ([32aff3e6](https://github.com/rsvim/rsvim/commit/32aff3e63ecf3f39c44cfb75f37f09c691a47575)) by @linrongbin16 ([#125](https://github.com/rsvim/rsvim/pull/125))

- *(js)* Refactor js runtime initialize (#134) ([6880a8f4](https://github.com/rsvim/rsvim/commit/6880a8f436868d9b8f668e5f2265372c9b4e3dbc)) by @linrongbin16 ([#134](https://github.com/rsvim/rsvim/pull/134))

- *(id)* Refactor incremental global id (#155) ([3434fd4a](https://github.com/rsvim/rsvim/commit/3434fd4a66e482ca3bb2ea15e3c6f0878f6913cd)) by @linrongbin16 ([#155](https://github.com/rsvim/rsvim/pull/155))

- *(err)* Refactor error code (#156) ([99327726](https://github.com/rsvim/rsvim/commit/99327726412ab1f116dc262ab184eea8a21b178c)) by @linrongbin16 ([#156](https://github.com/rsvim/rsvim/pull/156))

- *(js)* Use getter/setter API (#157) ([b9ba218c](https://github.com/rsvim/rsvim/commit/b9ba218cd5b8baf1ac0bfbae98cc505f9fc0a85f)) by @linrongbin16 ([#157](https://github.com/rsvim/rsvim/pull/157))

- *(ui)* Add reusable iframe (#174) ([a329ea44](https://github.com/rsvim/rsvim/commit/a329ea449d0ec65dbd8995a9d7b30d18474245ee)) by @linrongbin16 ([#174](https://github.com/rsvim/rsvim/pull/174))

- *(glovar)* Simplify "MUTEX_TIMEOUT" (#175) ([d99aee1f](https://github.com/rsvim/rsvim/commit/d99aee1f89224838f1317250b1d35a6d03dababa)) by @linrongbin16 ([#175](https://github.com/rsvim/rsvim/pull/175))

- *(opt)* Add constant default options (#176) ([cf7faa42](https://github.com/rsvim/rsvim/commit/cf7faa428773d004afdae65ba479c4accc4d4e9b)) by @linrongbin16 ([#176](https://github.com/rsvim/rsvim/pull/176))

- *(ui)* Move render logic out from window content (#177) ([4ef86b1f](https://github.com/rsvim/rsvim/commit/4ef86b1f85d6487f5ec431eb27e90619dbff60a3)) by @linrongbin16 ([#177](https://github.com/rsvim/rsvim/pull/177))

- *(opt)* Create both global/local options for window (#178) ([bff79708](https://github.com/rsvim/rsvim/commit/bff797088ef895272eb51c396ee43bd3880b5e84)) by @linrongbin16 ([#178](https://github.com/rsvim/rsvim/pull/178))

- *(win)* Access global options (#181) ([784d7778](https://github.com/rsvim/rsvim/commit/784d7778b5a545f1426a209db5c406782afae611)) by @linrongbin16 ([#181](https://github.com/rsvim/rsvim/pull/181))

- *(workspace)* Manage with workspace (#183) ([0da0e170](https://github.com/rsvim/rsvim/commit/0da0e17043b4bec3e6403839aba90661c59c4631)) by @linrongbin16 ([#183](https://github.com/rsvim/rsvim/pull/183))

- *(cli)* Remove "Cargo.toml" from version detect (#186) ([d550dbc4](https://github.com/rsvim/rsvim/commit/d550dbc4a98793e604724c4f935626ec7be8908f)) by @linrongbin16 ([#186](https://github.com/rsvim/rsvim/pull/186))

- *(js)* Merge built-in modules init into constructor (#194) ([52b3b022](https://github.com/rsvim/rsvim/commit/52b3b022d75becb238530702897584f696e29165)) by @linrongbin16 ([#194](https://github.com/rsvim/rsvim/pull/194))

- *(js)* Add basic v8 methods (#195) ([c9bde990](https://github.com/rsvim/rsvim/commit/c9bde990c00472e5b7157d15559f0b43a1148ff8)) by @linrongbin16 ([#195](https://github.com/rsvim/rsvim/pull/195))

- *(js)* Simplify built-in module as single file (#200) ([ec414499](https://github.com/rsvim/rsvim/commit/ec41449959907f7425e1aa6b5fcf67e83252634e)) by @linrongbin16 ([#200](https://github.com/rsvim/rsvim/pull/200))

- *(js)* Refactor context data index (#202) ([b15e932d](https://github.com/rsvim/rsvim/commit/b15e932df295be606ffd6ed0428ea77b850effc8)) by @linrongbin16 ([#202](https://github.com/rsvim/rsvim/pull/202))

## [0.1.1-alpha.5](https://github.com/rsvim/rsvim/compare/v0.1.1-alpha.3..v0.1.1-alpha.5) - 2024-09-13

[909f5f9f](https://github.com/rsvim/rsvim/commit/909f5f9f9ad6d4287d6a66371458c52a5a15b442)...[6e6c03c7](https://github.com/rsvim/rsvim/commit/6e6c03c7f828a5acbfae029370d061b5e907dd1e)

### <!-- 0 -->Features

- *(script)* Add js runtime (#109) ([da718832](https://github.com/rsvim/rsvim/commit/da718832eb301ad75080396de04fc5f989d59d0f)) by @linrongbin16 ([#109](https://github.com/rsvim/rsvim/pull/109))

## unreleased

### <!-- 0 -->Features

- *(poc)* Create async task queue (#92) ([02eeb427](https://github.com/rsvim/rsvim/commit/02eeb4276896ccb1a0920c5370c179a734d35a02)) by @linrongbin16 ([#92](https://github.com/rsvim/rsvim/pull/92))

- *(starting)* Input files (#100) ([0f341174](https://github.com/rsvim/rsvim/commit/0f341174121a701cd9e11b03e2c0656d48cd4ff8)) by @linrongbin16 ([#100](https://github.com/rsvim/rsvim/pull/100))


### <!-- 1 -->Bug Fixes

- *(buffer)* Fix file loading (#102) ([d39ea753](https://github.com/rsvim/rsvim/commit/d39ea753ebf5c7f11aa4a2206f8c3d665618522e)) by @linrongbin16 ([#102](https://github.com/rsvim/rsvim/pull/102))


### <!-- 3 -->Code Refactoring

- Move main to bin (#17) ([33fc0e4a](https://github.com/rsvim/rsvim/commit/33fc0e4a94cea41a1d9f7186903f95d28815f455)) by @linrongbin16 ([#17](https://github.com/rsvim/rsvim/pull/17))

- *(poc)* Refactor code, add docs (#71) ([6d9cc7f3](https://github.com/rsvim/rsvim/commit/6d9cc7f3cf673a3947e1261d48885acac8695e98)) by @linrongbin16 ([#71](https://github.com/rsvim/rsvim/pull/71))

- Rename 'Terminal' to 'Canvas' (#74) ([aafad5ca](https://github.com/rsvim/rsvim/commit/aafad5ca88205f41c81832a33d5101c077e4160b)) by @linrongbin16 ([#74](https://github.com/rsvim/rsvim/pull/74))

