# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1-alpha.10](https://github.com/rsvim/rsvim/compare/v0.1.1-alpha.9..0.1.1-alpha.10) - 2025-06-09

[d8577472](https://github.com/rsvim/rsvim/commit/d8577472c1180a5a4faa2b16a58182f3369b9dd3)...[b9366d72](https://github.com/rsvim/rsvim/commit/b9366d72418ab24dde07291c86063a1a3968262b)

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

<!-- generated by git-cliff -->
