## [0.1.3-alpha.2](https://github.com/rsvim/rsvim/compare/v0.1.2..0.1.3-alpha.2) - 2026-05-19

[79d7cd5c](https://github.com/rsvim/rsvim/commit/79d7cd5cc7a93d9633a6df603379e3b5714b79e5)...[5bb09218](https://github.com/rsvim/rsvim/commit/5bb092185dd209b569f5ee111185e2f25abd9411)

### <!-- 0 -->Features

- *(cli)* Provide v8 js engine version info in cli version (#210) ([9cb3cbc1](https://github.com/rsvim/rsvim/commit/9cb3cbc15e6f9f50452fb9a7eacc1305d71f8e60)) by @linrongbin16 ([#210](https://github.com/rsvim/rsvim/pull/210))

- *(ui)* Add viewport for better mapping buffer to window (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(buf)* Add "tabstop" local options for buffer (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(grapheme)* Add ascii control codes (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(unicode)* Add char display width calculation (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(ui)* Drop "breakat" option (#217) ([fbed93eb](https://github.com/rsvim/rsvim/commit/fbed93eb39483cbb5c0e5dce93159ded3c33b8be)) by @linrongbin16 ([#217](https://github.com/rsvim/rsvim/pull/217))

- *(buf)* Impl "new file" edit operation for buffers (#229) ([cad5833f](https://github.com/rsvim/rsvim/commit/cad5833f7c7f35c16d84747d98769542f1a21a1d)) by @linrongbin16 ([#229](https://github.com/rsvim/rsvim/pull/229))

- *(cursor)* Scroll buffer when cursor reaches window top/bottom (nowrap) (#322) ([31711f21](https://github.com/rsvim/rsvim/commit/31711f215fd19c60c16d98588a8a65e41a476b2e)) by @linrongbin16 ([#322](https://github.com/rsvim/rsvim/pull/322))

- *(cursor)* Impl cursor motion with buffer scrolling when reaches window border (#354) ([2045529d](https://github.com/rsvim/rsvim/commit/2045529d5104de14861d97a082a447c88887a6f5)) by @linrongbin16 ([#354](https://github.com/rsvim/rsvim/pull/354))

- *(cmdline)* Goto command-line ex mode (#482) ([d2eebc29](https://github.com/rsvim/rsvim/commit/d2eebc294d227e77440d488915433a4256d77811)) by @linrongbin16 ([#482](https://github.com/rsvim/rsvim/pull/482))

- *(v8)* Add default v8 flags (#509) ([13a3795a](https://github.com/rsvim/rsvim/commit/13a3795a7914699e637cf9064988958f8ae0e70b)) by @linrongbin16 ([#509](https://github.com/rsvim/rsvim/pull/509))

- *(exit)* Add "Rsvim.rt.exit()" API to quit editor process (#619) ([1e5eeb0c](https://github.com/rsvim/rsvim/commit/1e5eeb0c7cbb0f471ee1ad422cd1f3dd8612b272)) by @linrongbin16 ([#619](https://github.com/rsvim/rsvim/pull/619))

- *(buf)* Save editing changes to undo manager (#858) ([feb4b43b](https://github.com/rsvim/rsvim/commit/feb4b43bade3bd269855de871b9f01c82c7cfbee)) by @linrongbin16 ([#858](https://github.com/rsvim/rsvim/pull/858))

- *(hl)* Write captured highlight into canvas (#964) ([b835204f](https://github.com/rsvim/rsvim/commit/b835204f5a076785461347b77268397a6996d731)) by @linrongbin16 ([#964](https://github.com/rsvim/rsvim/pull/964))


### <!-- 1 -->Bug Fixes

- *(tui)* Recover terminal and save backtrace info when panic (#395) ([2299f0ef](https://github.com/rsvim/rsvim/commit/2299f0efdce62cc0013e5ae8bbc523d3698f9a6e)) by @linrongbin16 ([#395](https://github.com/rsvim/rsvim/pull/395))

- *(cmdline)* Fix cmdline ex mode go back to normal mode (#499) ([787cfe3c](https://github.com/rsvim/rsvim/commit/787cfe3c8b1d59ef8a301c8f3fc43c6b03d216b8)) by @linrongbin16 ([#499](https://github.com/rsvim/rsvim/pull/499))

- *(build)* Fix git repo not found when install with "cargo install" (#627) ([0edc36c7](https://github.com/rsvim/rsvim/commit/0edc36c74fa305d0d034aadb8147e9915be3fda1)) by @linrongbin16 ([#627](https://github.com/rsvim/rsvim/pull/627))

- *(ui)* Fix viewport search (#992) ([58a7ac61](https://github.com/rsvim/rsvim/commit/58a7ac619ec1227e454890725ce8a17c6a47251d)) by @linrongbin16 ([#992](https://github.com/rsvim/rsvim/pull/992))


### <!-- 2 -->Performance Improvements

- *(start)* Make snapshot v3 (#199) ([2f04e781](https://github.com/rsvim/rsvim/commit/2f04e7810b300937e1276f567764426a1c5eb69b)) by @linrongbin16 ([#199](https://github.com/rsvim/rsvim/pull/199))

- *(start)* Initialize built-in modules with snapshot (#205) ([73a92c0c](https://github.com/rsvim/rsvim/commit/73a92c0c3d20ac7c66a184987ce4024ad4918c7c)) by @linrongbin16 ([#205](https://github.com/rsvim/rsvim/pull/205))

- *(start)* Compress snapshot blob (#206) ([ac8420d1](https://github.com/rsvim/rsvim/commit/ac8420d17605aef8f975fdb5000774017a9f0384)) by @linrongbin16 ([#206](https://github.com/rsvim/rsvim/pull/206))

- *(size)* Move version string to build.rs to reduce binary size (#366) ([048daef0](https://github.com/rsvim/rsvim/commit/048daef050552c6ba8dbc70403d31c126cb5c943)) by @linrongbin16 ([#366](https://github.com/rsvim/rsvim/pull/366))

- *(cli)* Move to "fern" to reduce binary size (#567) ([fde0bb33](https://github.com/rsvim/rsvim/commit/fde0bb3384f1e706403126e024c2ffdd1a61ecd1)) by @linrongbin16 ([#567](https://github.com/rsvim/rsvim/pull/567))

- *(cli)* Move to "lexopt" to reduce binary size (#568) ([677005dc](https://github.com/rsvim/rsvim/commit/677005dc5f4b08bacc4c447ea592ffa8100f2be3)) by @linrongbin16 ([#568](https://github.com/rsvim/rsvim/pull/568))

- *(snapshot)* Disable compress on snapshot (#636) ([2538c56b](https://github.com/rsvim/rsvim/commit/2538c56b5b0bf05011683dc4f5c0300c958251f2)) by @linrongbin16 ([#636](https://github.com/rsvim/rsvim/pull/636))

- *(alloc)* Add optional custom allocator (#663) ([288e35b7](https://github.com/rsvim/rsvim/commit/288e35b76c0424c2fba7833d0e212635d5b2db48)) by @linrongbin16 ([#663](https://github.com/rsvim/rsvim/pull/663))

- *(msg)* Drain master/js messages to reduce memory allocation (#718) ([1b994234](https://github.com/rsvim/rsvim/commit/1b9942349a18b51b005d55f561963b4615e0661d)) by @linrongbin16 ([#718](https://github.com/rsvim/rsvim/pull/718))

- *(lock)* Switch to std mutex (#997) ([866e8794](https://github.com/rsvim/rsvim/commit/866e87940b4303b3dcafb4f463c41a358daf238c)) by @linrongbin16 ([#997](https://github.com/rsvim/rsvim/pull/997))


### <!-- 3 -->Code Refactoring

- *(workspace)* Manage with workspace (#183) ([0da0e170](https://github.com/rsvim/rsvim/commit/0da0e17043b4bec3e6403839aba90661c59c4631)) by @linrongbin16 ([#183](https://github.com/rsvim/rsvim/pull/183))

- *(cli)* Remove "Cargo.toml" from version detect (#186) ([d550dbc4](https://github.com/rsvim/rsvim/commit/d550dbc4a98793e604724c4f935626ec7be8908f)) by @linrongbin16 ([#186](https://github.com/rsvim/rsvim/pull/186))

- *(js)* Merge built-in modules init into constructor (#194) ([52b3b022](https://github.com/rsvim/rsvim/commit/52b3b022d75becb238530702897584f696e29165)) by @linrongbin16 ([#194](https://github.com/rsvim/rsvim/pull/194))

- *(js)* Refactor context data index (#202) ([b15e932d](https://github.com/rsvim/rsvim/commit/b15e932df295be606ffd6ed0428ea77b850effc8)) by @linrongbin16 ([#202](https://github.com/rsvim/rsvim/pull/202))

- *(js)* Clean up not used code (#215) ([1e0e0351](https://github.com/rsvim/rsvim/commit/1e0e0351f8b732299a34f365f2a190f6dc2ce873)) by @linrongbin16 ([#215](https://github.com/rsvim/rsvim/pull/215))

- *(cli)* Adjust initialize sequence in main loop (#215) ([1e0e0351](https://github.com/rsvim/rsvim/commit/1e0e0351f8b732299a34f365f2a190f6dc2ce873)) by @linrongbin16 ([#215](https://github.com/rsvim/rsvim/pull/215))

- *(ui)* Rename module "tree::util" to "tree::ptr" (#215) ([1e0e0351](https://github.com/rsvim/rsvim/commit/1e0e0351f8b732299a34f365f2a190f6dc2ce873)) by @linrongbin16 ([#215](https://github.com/rsvim/rsvim/pull/215))

- *(env)* Rename glovar to envar (#216) ([9744c693](https://github.com/rsvim/rsvim/commit/9744c693acf3fe0ce012c39f1d9dee4cebc46b82)) by @linrongbin16 ([#216](https://github.com/rsvim/rsvim/pull/216))

- *(prelude)* Add prelude and refactor other misc (#298) ([b4a8ab7b](https://github.com/rsvim/rsvim/commit/b4a8ab7beaf99cb6e6ce5b861763d7082ff6d0ed)) by @linrongbin16 ([#298](https://github.com/rsvim/rsvim/pull/298))

- *(cursor)* Rename `x`, `y` variable names to lines/chars/columns for readability (#362) ([d1b5eff7](https://github.com/rsvim/rsvim/commit/d1b5eff7db34befcf31442ff31b76cabbf4c23b8)) by @linrongbin16 ([#362](https://github.com/rsvim/rsvim/pull/362))

- *(cli)* Use 'PathBuf' for cli arguments (#557) ([71c72e7f](https://github.com/rsvim/rsvim/commit/71c72e7fae9bd7a9295163e8b756a38c9fa6f5f0)) by @linrongbin16 ([#557](https://github.com/rsvim/rsvim/pull/557))

- *(cli)* Add '--headless' cli option for mocking event loop (#571) ([96fb6045](https://github.com/rsvim/rsvim/commit/96fb6045f0d677260289adcf58c55a8baa9dd18d)) by @linrongbin16 ([#571](https://github.com/rsvim/rsvim/pull/571))

- *(cli)* Refactor special cli options version and help (#574) ([7909ff58](https://github.com/rsvim/rsvim/commit/7909ff58d6624815cce3bc83b3d8f7a360e32613)) by @linrongbin16 ([#574](https://github.com/rsvim/rsvim/pull/574))

- *(cli)* Add profile and git commit to version info (#611) ([a110259f](https://github.com/rsvim/rsvim/commit/a110259fc9110e88141787d250962b46b530d4bf)) by @linrongbin16 ([#611](https://github.com/rsvim/rsvim/pull/611))

- *(ui)* Refactor logical shape and actual shape calculation (#778) ([e5419cac](https://github.com/rsvim/rsvim/commit/e5419cac40c10fb4ceb2585592d8ed0a7ae166ca)) by @linrongbin16 ([#778](https://github.com/rsvim/rsvim/pull/778))

- *(lazy)* Replace std "LazyLock" with once_cell "Lazy" with parking_lot (#855) ([717a2d45](https://github.com/rsvim/rsvim/commit/717a2d459c8645e275233a618bb7094fff76fe58)) by @linrongbin16 ([#855](https://github.com/rsvim/rsvim/pull/855))

- *(cli)* Refactor cli version info (#954) ([85944a59](https://github.com/rsvim/rsvim/commit/85944a5990ee35b904cf9d3e9b89d2f09cf015f1)) by @linrongbin16 ([#954](https://github.com/rsvim/rsvim/pull/954))

- *(ui)* Refactor line processing logic when wrap=true & linebreak=false (#990) ([3891fc9d](https://github.com/rsvim/rsvim/commit/3891fc9d02855e7f06cfe44b5414b46ef6d8dcd2)) by @linrongbin16 ([#990](https://github.com/rsvim/rsvim/pull/990))

- *(cli)* Restore clap cli options (#1018) ([6a4fc24f](https://github.com/rsvim/rsvim/commit/6a4fc24ff74f12a745a3f9944f6c5276b1dbf2fb)) by @linrongbin16 ([#1018](https://github.com/rsvim/rsvim/pull/1018))

- *(color)* Split "syn"/"color" module, drop "parking_lot" from tokio (#1019) ([e0e4e4d0](https://github.com/rsvim/rsvim/commit/e0e4e4d015ebf9369203b1cacaa71ab1fcc0fb83)) by @linrongbin16 ([#1019](https://github.com/rsvim/rsvim/pull/1019))

- *(ui)* Stop hide and recover cursor if text is not changed (#1025) ([025857d5](https://github.com/rsvim/rsvim/commit/025857d5a57f13cd809185c70b842da8c1b619b4)) by @linrongbin16 ([#1025](https://github.com/rsvim/rsvim/pull/1025))


### <!-- 5 -->Testing

- *(evloop)* Add "run_with_mock_operations" for operation based loop run (#612) ([ded68501](https://github.com/rsvim/rsvim/commit/ded68501b5afec29db86d4117ef32ea232927da7)) by @linrongbin16 ([#612](https://github.com/rsvim/rsvim/pull/612))

