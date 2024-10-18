## [0.1.1-alpha.7](https://github.com/rsvim/rsvim/compare/v0.1.1-alpha.5..0.1.1-alpha.7) - 2024-10-18

[6e6c03c7](https://github.com/rsvim/rsvim/commit/6e6c03c7f828a5acbfae029370d061b5e907dd1e)...[cea42e7a](https://github.com/rsvim/rsvim/commit/cea42e7af0bc1205c4427918eaaf457e2e7d80b3)

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

