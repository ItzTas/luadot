# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [v0.3.0](https://github.com/ItzTas/luadot/compare/v0.3.0-nightly.1..v0.3.0) - 2026-09-04

- - -

## [v0.3.0-nightly.1](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.19..v0.3.0-nightly.1) - 2026-09-04
#### Miscellaneous Chores
- (**version**) v0.3.0-nightly.1 [skip ci] - ([4524e79](https://github.com/ItzTas/luadot/commit/4524e79327f9e0475f6368e4f771bbd06075a7cf)) - Tales Sabini
- merge main into nightly - ([3f8a997](https://github.com/ItzTas/luadot/commit/3f8a997783815c90772dabaa7536f70e96cfd1ed)) - luadot

- - -

## [v0.1.0-nightly.19](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.18..v0.1.0-nightly.19) - 2026-09-04
#### Bug Fixes
- (**ci**) give the binaries job the memory it needs - ([6dc1ff2](https://github.com/ItzTas/luadot/commit/6dc1ff2d386893b88e54968aff5be779f827200d)) - luadot
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.19 [skip ci] - ([97252bc](https://github.com/ItzTas/luadot/commit/97252bc71dcd54effd2da6a4787b142973a56a8d)) - Tales Sabini

- - -

## [v0.1.0-nightly.18](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.17..v0.1.0-nightly.18) - 2026-09-03
#### Features
- (**cli**) add the relink command - ([427381a](https://github.com/ItzTas/luadot/commit/427381ab8f2bb7a8212bd9408acbbd2f576213b2)) - luadot
#### Documentation
- document relink, diff output and template placement - ([73846cf](https://github.com/ItzTas/luadot/commit/73846cfc8dcd90a074b442e61d61ec7e913e038b)) - luadot
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.18 [skip ci] - ([366a5b8](https://github.com/ItzTas/luadot/commit/366a5b80fcc3503907c2c3afb87cc0e3903248ab)) - Tales Sabini
- regenerate the completions and ld.lua - ([a376812](https://github.com/ItzTas/luadot/commit/a376812d3aafc6ef70afa665dd383ec098c125a9)) - luadot
#### Style
- (**ld**) format the alt sections param kind - ([1c6a292](https://github.com/ItzTas/luadot/commit/1c6a292caf0d02eed6516e196ba77d00d858b2e7)) - luadot

- - -

## [v0.1.0-nightly.17](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.16..v0.1.0-nightly.17) - 2026-09-03
#### Features
- (**apply**) place a template's source files instead of skipping them - ([83a78a4](https://github.com/ItzTas/luadot/commit/83a78a49ebdb3581614ebd3ef2073b50b0c9f015)) - luadot
#### Documentation
- describe template source files as managed like any other - ([bb406a8](https://github.com/ItzTas/luadot/commit/bb406a8321c27e01dbef8c51320df118a9ea5e15)) - luadot
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.17 [skip ci] - ([067e48d](https://github.com/ItzTas/luadot/commit/067e48d254b3bde0bf544412a7ae4899900e3af9)) - Tales Sabini

- - -

## [v0.1.0-nightly.16](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.15..v0.1.0-nightly.16) - 2026-08-29
#### Features
- (**cli**) add -u/--unchanged to list files already in sync - ([6d5e48d](https://github.com/ItzTas/luadot/commit/6d5e48d37a2b54d8fbce8b49131a6fd357d98827)) - luadot
- (**opt**) add the hints option - ([2797497](https://github.com/ItzTas/luadot/commit/2797497f7c0683fe56f097f42ad481132dbe031c)) - luadot
- (**take**) take everything the repository holds with no path - ([9c05ae3](https://github.com/ItzTas/luadot/commit/9c05ae33436e3db6b49dc6f78a04328b8f624a09)) - luadot
#### Bug Fixes
- (**config**) refuse a config.lua others can write or own - ([31fcfdf](https://github.com/ItzTas/luadot/commit/31fcfdff0976bb6853e816c92b89dc2afe3e7e57)) - luadot
- (**crypt**) leave decrypted plaintext readable by its owner alone - ([09b2d86](https://github.com/ItzTas/luadot/commit/09b2d865ac24d9fa1ac785b335985da97617ecf1)) - luadot
- (**files**) never place mode or owner through a symlink - ([85bd7d3](https://github.com/ItzTas/luadot/commit/85bd7d3663b05c83d80b3b411d20eebb211771f2)) - luadot
- (**ld**) export HINTS from the ld module - ([740c6eb](https://github.com/ItzTas/luadot/commit/740c6eb1596c30aed50a227b03d833824449270d)) - luadot
- (**paths**) confine the xdg directories to the home - ([eeff0df](https://github.com/ItzTas/luadot/commit/eeff0dfca5d7b71d1ebdee25aac4e272db215348)) - luadot
- (**restore**) keep restoration inside what luadot manages - ([b181d83](https://github.com/ItzTas/luadot/commit/b181d837cc97ebe2deb07dc25acb4362d3bad8d8)) - luadot
- (**state**) keep the data directory and the state file private - ([7440379](https://github.com/ItzTas/luadot/commit/7440379aef2c483d1191e188d60debacfe4559a5)) - luadot
#### Tests
- (**alt**) drop the sys reference from the expand fixture - ([2aca696](https://github.com/ItzTas/luadot/commit/2aca696eedeaeefb0c9964bbac7b6ece54f02ffc)) - luadot
- (**cli**) record the plaintext mode without gnu-only stat - ([5346a22](https://github.com/ItzTas/luadot/commit/5346a2206d2f24da7b34d370fd11c2f7463e6f84)) - luadot
- (**on**) assert every command reaches its function and its definition - ([2551fd5](https://github.com/ItzTas/luadot/commit/2551fd511afff5c3004120fb79efccbd87f064fa)) - luadot
#### Refactoring
- (**files**) add link_at, dedupe the symlink check in add, mv and rm - ([a271cf4](https://github.com/ItzTas/luadot/commit/a271cf4d679aebeef09d65e3714bb91a0e16149b)) - luadot
- (**json**) decode and encode through mlua's LuaSerdeExt - ([9baf965](https://github.com/ItzTas/luadot/commit/9baf96554139c3183bb797c6e069c6dcb3548af2)) - luadot
- (**ld**) move meta descriptions out of constants into describe - ([e69ed1d](https://github.com/ItzTas/luadot/commit/e69ed1d0b0b78bac89b4f95b9ec5fea77736240c)) - luadot
- (**ld**) remove the sys namespace - ([46ef030](https://github.com/ItzTas/luadot/commit/46ef03079503c5969ba986fc83cd34e264a13bd1)) - luadot
- (**on**) derive the command list from the enum - ([6be4e6a](https://github.com/ItzTas/luadot/commit/6be4e6a56ab99d7a05e3f182d1210707fb7f7af7)) - luadot
#### Miscellaneous Chores
- (**cli**) ship the git-forwarding completion scripts as standalone files - ([e31140c](https://github.com/ItzTas/luadot/commit/e31140cbeaf0a2413bb88758e422c256159c22cb)) - luadot
- (**ld**) warn when a mode rule asks for setuid or setgid - ([a357de3](https://github.com/ItzTas/luadot/commit/a357de3dc16077cb7b38be8365e3cd1be98ea428)) - luadot
- (**meta**) regenerate ld.lua for the sys removal and hints option - ([53b198f](https://github.com/ItzTas/luadot/commit/53b198f96a9e4f262626ac8e1b3daaca4136da2e)) - luadot
- (**version**) v0.1.0-nightly.16 [skip ci] - ([8e68f67](https://github.com/ItzTas/luadot/commit/8e68f67e25acb7e6e2732f41a55ae96b8df067ce)) - Tales Sabini
- ignore the omo run-continuation directory - ([eb386aa](https://github.com/ItzTas/luadot/commit/eb386aabd7dfba83811ca9cd1fa1652b946f1fc2)) - luadot

- - -

## [v0.1.0-nightly.15](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.14..v0.1.0-nightly.15) - 2026-08-25
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.15 [skip ci] - ([1073fba](https://github.com/ItzTas/luadot/commit/1073fbaf74f7e55c2d9f8c77da76cf873abbb82d)) - Tales Sabini

- - -

## [v0.1.0-nightly.14](https://github.com/ItzTas/luadot/compare/e4f1dc7e8e0835400d53c337b6e5afa62604e3e0..v0.1.0-nightly.14) - 2026-08-25
#### Features
- (**cli**) add whole-directory adoption and a take command - ([226b4cf](https://github.com/ItzTas/luadot/commit/226b4cf5f43778acc08b32a845a5055e7f844501)) - luadot
#### Bug Fixes
- (**changelog**) keep the releases below a partial render - ([e4f1dc7](https://github.com/ItzTas/luadot/commit/e4f1dc7e8e0835400d53c337b6e5afa62604e3e0)) - luadot
#### Documentation
- document track, whole directories, and take - ([8285228](https://github.com/ItzTas/luadot/commit/8285228898883bce22bd4a2ad5e5636f3389fa5e)) - luadot
#### Tests
- trim redundant words from test names - ([923f829](https://github.com/ItzTas/luadot/commit/923f829fbf5ec99a412ca2667a78a19f712047cf)) - luadot
#### Miscellaneous Chores
- (**meta**) regenerate ld.lua doc comments - ([de2aaa6](https://github.com/ItzTas/luadot/commit/de2aaa62459e4b97ea091d7075c49c6823304984)) - luadot
- (**version**) v0.1.0-nightly.14 [skip ci] - ([70d3d0c](https://github.com/ItzTas/luadot/commit/70d3d0cd91d346a7b472c33593ba640aa427e426)) - Tales Sabini

- - -

## [v0.2.1](https://github.com/ItzTas/luadot/compare/286da8c0c67f00dde4b27a070a24f8773d0d4229..v0.2.1) - 2026-08-25
#### Bug Fixes
- (**changelog**) put back the releases the truncation dropped - ([64a36a5](https://github.com/ItzTas/luadot/commit/64a36a55bfcbb56100c709aaf701e67c7627b303)) - luadot
- (**changelog**) keep the releases below a partial render - ([286da8c](https://github.com/ItzTas/luadot/commit/286da8c0c67f00dde4b27a070a24f8773d0d4229)) - luadot

- - -

## [v0.2.0](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.13..v0.2.0) - 2026-08-25
#### Miscellaneous Chores
- bring main up to nightly - ([d8d81b2](https://github.com/ItzTas/luadot/commit/d8d81b2a35ef1c0bec37899953885639bad20445)) - luadot

- - -

## [v0.1.0-nightly.13](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.12..v0.1.0-nightly.13) - 2026-08-25
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.13 [skip ci] - ([6b0bc05](https://github.com/ItzTas/luadot/commit/6b0bc053bbb2c45548225d1c739bb1f63f8ef858)) - Tales Sabini
- remove the file a test commit left in the repository - ([99736d2](https://github.com/ItzTas/luadot/commit/99736d2671bcea423e39d0412f1e3df785df88fa)) - luadot

- - -

## [v0.1.0-nightly.12](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.11..v0.1.0-nightly.12) - 2026-08-24
#### Features
- (**cli**) move a managed file with luadot mv - ([34a6a53](https://github.com/ItzTas/luadot/commit/34a6a530705bcc61aa5c5374f3cf8a83ae44013e)) - Tales
#### Bug Fixes
- (**changelog**) drop the releases the merge brought back - ([2548b80](https://github.com/ItzTas/luadot/commit/2548b8011942017d4f4f23cd82c94cd4807a01a9)) - Tales
- (**changelog**) tell releases apart by version, not by rendered line - ([4e6c116](https://github.com/ItzTas/luadot/commit/4e6c116a7a227937e001130335ed65a3028c73a7)) - Tales
- (**changelog**) drop the releases every bump repeats - ([7ac6ef3](https://github.com/ItzTas/luadot/commit/7ac6ef3464e13935bd47578cd1e61b9492b7a330)) - Tales
#### Tests
- (**git**) run the panic-hook test in its own process - ([694c8c2](https://github.com/ItzTas/luadot/commit/694c8c256147600c66b752ab3beb6d39df7474a6)) - Tales
#### Continuous Integration
- publish the release assets to github - ([9dd58f0](https://github.com/ItzTas/luadot/commit/9dd58f0672229e05d9754e2cf3af606fd77912e3)) - Tales
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.12 [skip ci] - ([a84cf1e](https://github.com/ItzTas/luadot/commit/a84cf1ebbbae0e5070eb820ae9705237977b4586)) - Tales Sabini

- - -

## [v0.1.0-nightly.11](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.10..v0.1.0-nightly.11) - 2026-08-22
#### Features
- (**cli**) show restore's files per backup and created/replaced outcomes - ([a6505a8](https://github.com/ItzTas/luadot/commit/a6505a8e93396183c9c2fc943fe6a9f1ec43d974)) - Tales
- (**cli**) report each file add lands and how many - ([890a581](https://github.com/ItzTas/luadot/commit/890a581fbd8f6491019d886fdf3571c6a1d2147c)) - Tales
- (**cli**) make doc without a call list every one - ([40170eb](https://github.com/ItzTas/luadot/commit/40170eb0f9f07372bfd2ce85972a51f37ff6e36c)) - Tales
- (**cli**) write a starter config on init and before first edit - ([7566eba](https://github.com/ItzTas/luadot/commit/7566eba3d017e0fa89c54f955beb9c33a002908f)) - Tales
#### Bug Fixes
- (**build**) only install git hooks path when the manifest dir is the repo root - ([23f4115](https://github.com/ItzTas/luadot/commit/23f411521cbe865a861f3d9b7de890c292d71c19)) - Tales
- (**cli**) suggest the nearest command or task for an unknown name - ([c86bc4a](https://github.com/ItzTas/luadot/commit/c86bc4ab7135ab484421d3e6efef5b5e7f2efac7)) - Tales
- (**files**) pass the option separator before the owner to chown - ([6ef663a](https://github.com/ItzTas/luadot/commit/6ef663abea3b843a71ddb84126829357ad8bb650)) - Tales
- (**lua**) keep the surface list out of a build without the meta feature - ([aee25c1](https://github.com/ItzTas/luadot/commit/aee25c1c52343d236b6c9d3bc1ba96b7a8ebaf78)) - Tales
#### Documentation
- cover restore's per-file outcomes, doc listing and starter config - ([c7a2ca0](https://github.com/ItzTas/luadot/commit/c7a2ca0216e1b16f4d3d1593a638fbb204decdca)) - Tales
- document plugin support - ([046daa6](https://github.com/ItzTas/luadot/commit/046daa627b652573c67aedfe4c211686a1b4be9f)) - Tales
#### Tests
- (**cli**) cover the starter config, doc listing and restore/add reporting - ([1258ed1](https://github.com/ItzTas/luadot/commit/1258ed19b89e718bdf05d966bfece88142089c04)) - Tales
- (**cli**) take the unfinished doc and restore assertions out of the branch - ([51c6049](https://github.com/ItzTas/luadot/commit/51c60497a60e057bfbdfc1577f153ed15807ab94)) - Tales
- (**cli**) feed the fake age through stdin so BSD base64 takes it - ([64b767f](https://github.com/ItzTas/luadot/commit/64b767f892493a496fdb2a264daffbdd1335789e)) - Tales
- (**cli**) cover a plugin registered from the configuration end to end - ([6339d33](https://github.com/ItzTas/luadot/commit/6339d33cbab086a69164894eb4e4733b0bdc671e)) - Tales
#### Continuous Integration
- publish releases to crates.io - ([3c04b7c](https://github.com/ItzTas/luadot/commit/3c04b7ce81044747254a79c9cb5ec9b18ef50ff4)) - Tales
- build and test on macOS - ([d1ed984](https://github.com/ItzTas/luadot/commit/d1ed9844b16f086e95009f37034a01511f4c105c)) - Tales
#### Miscellaneous Chores
- (**cli**) point the editor at plugin definitions - ([7368e64](https://github.com/ItzTas/luadot/commit/7368e64877f7f91465f2fb8d5eea8823237d2661)) - Tales
- (**cli**) describe the calls a plugin registers - ([1379661](https://github.com/ItzTas/luadot/commit/1379661c6ac56a2594668c0264fae2ed59b42cc7)) - Tales
- (**lua**) name the surface the script is running on - ([96d736c](https://github.com/ItzTas/luadot/commit/96d736c36ed0dc28ab436fc80097cc2294419863)) - Tales
- (**version**) v0.1.0-nightly.11 [skip ci] - ([201ae7a](https://github.com/ItzTas/luadot/commit/201ae7a8bb4c7664651aa8bfcb98bf72d8238bee)) - github push repo
- conflitos consertados - ([3c85572](https://github.com/ItzTas/luadot/commit/3c85572f1e1d5261ec1090194a325e694b11abaa)) - Tales
- stop tracking local .cargo registry cache - ([01d1842](https://github.com/ItzTas/luadot/commit/01d1842e45f886fdc17e04733181abc4061f32d6)) - Tales

- - -

## [v0.1.0-nightly.10](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.9..v0.1.0-nightly.10) - 2026-08-22
#### Features
- (**cli**) run the tasks the configuration registers - ([be016bf](https://github.com/ItzTas/luadot/commit/be016bfd51a0104172346c941f31a26c1d2011e2)) - Tales
#### Bug Fixes
- (**lua**) run every hook registered for a command - ([f734e93](https://github.com/ItzTas/luadot/commit/f734e936e186ee8968b265fdd8702db63fb45243)) - Tales
#### Documentation
- document the rules relocation and the definitions directory change - ([529bcde](https://github.com/ItzTas/luadot/commit/529bcde190fb4ee46eebbd7e616eb026a7f7ea79)) - Tales
#### Tests
- (**cli**) cover the rules relocation and definitions change end to end, drop superseded cases - ([5b2dad8](https://github.com/ItzTas/luadot/commit/5b2dad84a315b3a8e4298b25cd903b8bdf5313e4)) - Tales
- prune redundant and duplicate unit test coverage - ([d87d1bc](https://github.com/ItzTas/luadot/commit/d87d1bcc3579049d6ec5d22f016b2bdc43142675)) - Tales
#### Refactoring
- (**cli**) write the lua-language-server definitions only into the configuration directory - ([9233af3](https://github.com/ItzTas/luadot/commit/9233af3d481a25517ada99b0bb79c0c356d1304a)) - Tales
- (**git**) relocate the repository's ignore and attribute rules under .git/info - ([97958e4](https://github.com/ItzTas/luadot/commit/97958e4ee899ae0cb0101aaa3ed7535a3d7f25f6)) - Tales
#### Miscellaneous Chores
- (**git**) clone and run git outside the managed repository - ([64375da](https://github.com/ItzTas/luadot/commit/64375da89a754f4fcc3c691e0851aec6f408ce5f)) - Tales
- (**lua**) reach the filesystem from ld.fs - ([2c2ee4a](https://github.com/ItzTas/luadot/commit/2c2ee4aa7639ac066354fbdd1782bc74a9ba3c2b)) - Tales
- (**lua**) encode and decode JSON from ld.json - ([c2d8a19](https://github.com/ItzTas/luadot/commit/c2d8a19135355d8c6e5d6c2d7867d1c8d7d3ca2b)) - Tales
- (**lua**) answer the data directory in ld.path - ([37abd04](https://github.com/ItzTas/luadot/commit/37abd04334cb0321f44862f1b139e1bd7f061c14)) - Tales
- (**lua**) add module paths that outlive the configuration - ([ffdc25b](https://github.com/ItzTas/luadot/commit/ffdc25b36b60427e18b1c126903a7b011b889002)) - Tales
- (**version**) v0.1.0-nightly.10 [skip ci] - ([1027f03](https://github.com/ItzTas/luadot/commit/1027f0353dfa8b22beb51867d2ffc178fd53067c)) - Tales Sabini

- - -

## [v0.1.0-nightly.9](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.8..v0.1.0-nightly.9) - 2026-08-21
#### Features
- (**git**) clean up gix's temp files on an interrupt or a panic - ([1e35369](https://github.com/ItzTas/luadot/commit/1e353691bc7323623d3ffd9662153bdce982db83)) - Tales
#### Bug Fixes
- (**cli**) offer the lua-language-server definitions after init and clone - ([e8c2667](https://github.com/ItzTas/luadot/commit/e8c2667a2af3adaf2a9dfe84569a5982aa6aab7b)) - Tales
- (**crypt**) make Ahead::width's core count an explicit argument - ([f1dd863](https://github.com/ItzTas/luadot/commit/f1dd86366aae831a2dc8304cd8ebf190793bca9c)) - Tales
- (**git**) report a failed interrupt guard instead of ignoring it - ([9c4a999](https://github.com/ItzTas/luadot/commit/9c4a9993e7e1dc81747d6cb5aeb19a444ad026c7)) - Tales
#### Documentation
- document the flat layout, Git LFS, command hooks, and definitions refresh - ([b6ae025](https://github.com/ItzTas/luadot/commit/b6ae025e6806f4cf5de98746438a0a7fbf3a0d09)) - Tales
#### Tests
- (**cli**) cover the flat layout, LFS, hooks, and definitions refresh end to end - ([870c26b](https://github.com/ItzTas/luadot/commit/870c26be2b0feae4a264644d3c09a2256a9a3102)) - Tales
- (**git**) cover the panic hook releasing held tempfile locks - ([7e4ec60](https://github.com/ItzTas/luadot/commit/7e4ec60d2b0f5576494f34690308c88d5ef4988a)) - Tales
#### Refactoring
- (**lua**) collapse ld describe() functions onto Collect::namespace - ([56614d8](https://github.com/ItzTas/luadot/commit/56614d80ec08397862612a159f3fe5f7a605c6c3)) - Tales
- ![BREAKING](https://img.shields.io/badge/BREAKING-red) flatten the managed layout onto unprivileged Placement, add Git LFS and before/after hooks - ([46e840e](https://github.com/ItzTas/luadot/commit/46e840e82a42c8c8850d26da3ce9c947bda11c5b)) - Tales
#### Miscellaneous Chores
- (**ci**) audit dependencies with cargo-deny and check vendored/generated sources stay current - ([1bd5b5b](https://github.com/ItzTas/luadot/commit/1bd5b5b5e1c44a19f592d14e494815ddc2838061)) - Tales
- (**version**) v0.1.0-nightly.9 [skip ci] - ([69e236d](https://github.com/ItzTas/luadot/commit/69e236d8f53f2c727022da242d68d81c54cd2bcd)) - Tales Sabini

- - -

## [v0.1.0-nightly.8](https://github.com/ItzTas/luadot/compare/6bce4afaf739d2b95b24ae8d0dfc6d963ced8dd8..v0.1.0-nightly.8) - 2026-08-21
#### Features
- (**cli**) write editor definitions for the configuration - ([201b8b4](https://github.com/ItzTas/luadot/commit/201b8b4d195fc754f29845b834c74e1e2b2b754a)) - Tales
- (**cli**) add `luadot doc` and `luadot man` - ([5120ba2](https://github.com/ItzTas/luadot/commit/5120ba2bb2200e3ffdde88d84190fc81aa7fdc64)) - Tales
- (**lua**) let ld option and class calls take effect from any script - ([0a8f51a](https://github.com/ItzTas/luadot/commit/0a8f51a67f51a24a569daed73f10f19b52c034cd)) - Tales
- (**lua**) let ld.alt and ld.opt calls run from any script - ([c492253](https://github.com/ItzTas/luadot/commit/c492253d4bc77043a74d5e7344d454e971ee3184)) - Tales
#### Bug Fixes
- (**bench**) thread the shared configuration through the template bench - ([44d73da](https://github.com/ItzTas/luadot/commit/44d73da325fd7c8bf8487d54d63877da7addd7de)) - Tales
- (**lua**) satisfy clippy on the shared-configuration signatures - ([400f3ad](https://github.com/ItzTas/luadot/commit/400f3ad20c583602566168e9ebda7e24c03ebf2c)) - Tales
#### Documentation
- mention the settings a new template gets - ([d781b28](https://github.com/ItzTas/luadot/commit/d781b28ec7b860045273e7322a064b3263289472)) - Tales
- document editor support - ([ed4b7ab](https://github.com/ItzTas/luadot/commit/ed4b7abb1ceb97ed273c96f53f49070f47ea0d95)) - Tales
- trim explanations already covered elsewhere - ([186f45c](https://github.com/ItzTas/luadot/commit/186f45c59155216a23b2a5617f23ea65a9413d14)) - Tales
#### Tests
- (**cli**) check a new template carries its language server settings - ([28b62e2](https://github.com/ItzTas/luadot/commit/28b62e27a6f4d087250767d0969412d2198afb81)) - Tales
- (**cli**) drive the editor definitions end to end - ([bb93a3d](https://github.com/ItzTas/luadot/commit/bb93a3db4933fd16b47575e1b174076309d64767)) - Tales
#### Miscellaneous Chores
- (**aur**) drop debug info from AUR packages - ([bee1b60](https://github.com/ItzTas/luadot/commit/bee1b60455989c98a73ca772dc97abfdd90f1578)) - Tales
- (**ci**) check the editor definitions are current - ([0c72ba4](https://github.com/ItzTas/luadot/commit/0c72ba45f7a0c36464b51f456f15357df1ea12b5)) - Tales
- (**cli**) point a new template at the editor definitions - ([282e859](https://github.com/ItzTas/luadot/commit/282e8599eee98bac5f57bfc5f9d05554c17563ff)) - Tales
- (**deps**) add tealr behind the meta feature - ([d32846a](https://github.com/ItzTas/luadot/commit/d32846a88a5a8c0023e4aaaf1acdb1ec4317d5d0)) - Tales
- (**deps**) add clap_mangen - ([02fd048](https://github.com/ItzTas/luadot/commit/02fd0480a21841ee0f4d4322d4dcdc3c5bc16aee)) - Tales
- (**lua**) render the ld definitions as lua-language-server meta - ([62da696](https://github.com/ItzTas/luadot/commit/62da6966c7fd76fe53c3c68f213f51b63c50dc0b)) - Tales
- (**lua**) describe the ld surface with tealr - ([3e6ddd3](https://github.com/ItzTas/luadot/commit/3e6ddd3830317f69e14d521db0e0c667964f0932)) - Tales
- (**lua**) collect the rule keys into one array and refuse the rest - ([bd82a4b](https://github.com/ItzTas/luadot/commit/bd82a4bde9ab9b3629b88d0de97eeb91a9b51b2c)) - Tales
- (**version**) v0.1.0-nightly.8 [skip ci] - ([d8a15df](https://github.com/ItzTas/luadot/commit/d8a15df3d3588ed52fd8721ff26bd76de8f1a9a2)) - Tales Sabini
- require the humanizer skill for documentation edits - ([6bce4af](https://github.com/ItzTas/luadot/commit/6bce4afaf739d2b95b24ae8d0dfc6d963ced8dd8)) - Tales

- - -


- - -

## [v0.1.0](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.7..v0.1.0) - 2026-08-20

- - -

## [v0.1.0-nightly.7](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.6..v0.1.0-nightly.7) - 2026-08-20
#### Bug Fixes
- (**aur**) fetch release assets from the tag's registry path - ([ca16295](https://github.com/ItzTas/luadot/commit/ca16295c6def2ede224b3f61cb8dc2ff10a99f9f)) - Tales
#### Documentation
- open the README with a summary of the project - ([0521300](https://github.com/ItzTas/luadot/commit/0521300b360ce33d53137e7a826f2efe243af276)) - Tales
- split the README into per-topic pages - ([38696c7](https://github.com/ItzTas/luadot/commit/38696c7584df39ce320b86c0545dce697b41e391)) - Tales
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.7 [skip ci] - ([a5434b2](https://github.com/ItzTas/luadot/commit/a5434b2c26157c12a30b6318f7bea859e7afe024)) - Tales Sabini

- - -

## [v0.1.0-nightly.6](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.5..v0.1.0-nightly.6) - 2026-08-20
#### Bug Fixes
- (**ci**) upload release assets with the job token - ([6445491](https://github.com/ItzTas/luadot/commit/6445491597e552569e89f18f5213e963ef19cee3)) - Tales
#### Documentation
- move maintainer notes into internal/ - ([2a136d5](https://github.com/ItzTas/luadot/commit/2a136d52d6a4e945eaa2e791d7aa1c0bb2300971)) - Tales
- remove readme content - ([684d9b4](https://github.com/ItzTas/luadot/commit/684d9b42ad1a29623fd4862e8545e4173f0c0a7f)) - Tales
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.6 [skip ci] - ([46f77c0](https://github.com/ItzTas/luadot/commit/46f77c0fe8e170208572a6628999ecbeff9ebe3d)) - github push repo

- - -

## [v0.1.0-nightly.5](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.4..v0.1.0-nightly.5) - 2026-08-20
#### Features
- (**lua**) add lpeg module support - ([598a188](https://github.com/ItzTas/luadot/commit/598a188ed712663161a34406a72d7f1e6bcd4515)) - Tales
#### Bug Fixes
- (**lua**) let a global autopush imply the autocommit - ([0917ea5](https://github.com/ItzTas/luadot/commit/0917ea5d637c93a6b1c757c48a395f583cf9c066)) - Tales
#### Documentation
- (**readme**) document the identity taking a path or a command - ([7cda853](https://github.com/ItzTas/luadot/commit/7cda853e3b3982ca5f755f9599e04934ef2df88a)) - Tales
- (**readme**) document the crypt lock - ([733b80c](https://github.com/ItzTas/luadot/commit/733b80c2e0b0fa75db75bea22e4049507c63b09c)) - Tales
- (**vendoring**) explain the vendored sources and how to update them - ([15c28e2](https://github.com/ItzTas/luadot/commit/15c28e2b93a89e81b8a139af018e18a8738ee7cc)) - Tales
#### Build system
- vendor the lpeg sources instead of downloading them - ([984328f](https://github.com/ItzTas/luadot/commit/984328f19ec989b86f2ad84b6c0b62bf44efb93d)) - Tales
#### Continuous Integration
- stop caching target/ in the test job - ([a3e844e](https://github.com/ItzTas/luadot/commit/a3e844e647363c9b1f55e331292e80bcdd3564b2)) - Tales
- split the pipeline into separate workflow files - ([ab418cd](https://github.com/ItzTas/luadot/commit/ab418cd4014413405c4d4682d03e52c101b02b1f)) - Tales
- check the vendored tree on every push - ([277a4ee](https://github.com/ItzTas/luadot/commit/277a4eee6be66065e613b1b41e999e5d4613b5a4)) - Tales
#### Refactoring
- ![BREAKING](https://img.shields.io/badge/BREAKING-red) (**crypt**) fold identity_command into the identity - ([9d6282d](https://github.com/ItzTas/luadot/commit/9d6282dbb6d2fe7f13554e6b854baafc455cec5b)) - Tales
- ![BREAKING](https://img.shields.io/badge/BREAKING-red) (**crypt**) fold the lock into one call - ([3fd3bf7](https://github.com/ItzTas/luadot/commit/3fd3bf763b72388fd6fc7e2d28d33fc7e2fcf2e4)) - Tales
- rework the cli, lua, git and output modules - ([8d11a19](https://github.com/ItzTas/luadot/commit/8d11a193ddadd8531281d97b97cb896d48a39297)) - Tales
- restructure the cli, lua and output modules - ([1ac1364](https://github.com/ItzTas/luadot/commit/1ac1364ec86c8fd9f505a0b11b5d4d906c26762e)) - Tales
#### Miscellaneous Chores
- (**nix**) package luadot with a nix flake - ([3ef8619](https://github.com/ItzTas/luadot/commit/3ef8619f011b846fbb2de4eef9680d9d0417731f)) - Tales
- (**release**) build and publish the release assets from CI - ([8bc0c41](https://github.com/ItzTas/luadot/commit/8bc0c41d20b3cd23c994e4df01d32313f8387bd0)) - Tales
- (**version**) v0.1.0-nightly.5 [skip ci] - ([07b9f4f](https://github.com/ItzTas/luadot/commit/07b9f4f822a4ccef33c5975e39b6d2201b42ec46)) - Tales Sabini

- - -

## [v0.1.0-nightly.4](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.3..v0.1.0-nightly.4) - 2026-08-18
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.4 [skip ci] - ([84c239c](https://github.com/ItzTas/luadot/commit/84c239ce74a095996d368554f4232e3611e220bf)) - github push repo

- - -

## [v0.1.0-nightly.3](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.2..v0.1.0-nightly.3) - 2026-08-18
#### Features
- (**cli**) add the init command - ([fdf65d8](https://github.com/ItzTas/luadot/commit/fdf65d8aa860873361541bb9b7782cac76b88e5d)) - Tales
#### Documentation
- (**design-system**) rule out tests another test already covers - ([5033caf](https://github.com/ItzTas/luadot/commit/5033caf5f31d2cdc1c7bbaaf8c68eb1c86bac426)) - Tales
- (**readme**) document the init command - ([fc9b747](https://github.com/ItzTas/luadot/commit/fc9b74705f4ab62c2a4d996b0cbd7ae868f1080b)) - Tales
#### Tests
- drop tests already covered by other tests - ([e38fe5d](https://github.com/ItzTas/luadot/commit/e38fe5d34b9daae88251934903207655b793748b)) - Tales
#### Refactoring
- share the repository destination and empty-directory checks - ([52ebf03](https://github.com/ItzTas/luadot/commit/52ebf034c77a469cf4315213a3eadad30e1f4947)) - Tales
#### Miscellaneous Chores
- (**version**) v0.1.0-nightly.3 [skip ci] - ([cb59c68](https://github.com/ItzTas/luadot/commit/cb59c688cc60b511ab4a99d9723a2adc99153f2b)) - github push repo

- - -

## [v0.1.0-nightly.2](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.1..v0.1.0-nightly.2) - 2026-08-18
#### Miscellaneous Chores
- (**aur**) drop check() from PKGBUILD templates (#7) - ([3373052](https://github.com/ItzTas/luadot/commit/3373052ddc06aeecf8a67ef5869f31697ac05c92)) - Tales Sabini
- (**version**) v0.1.0-nightly.2 [skip ci] - ([18aeec8](https://github.com/ItzTas/luadot/commit/18aeec8ac0532d63ef70fdd284ff086d4189cc0e)) - github push repo

- - -

## [v0.1.0-nightly.1](https://github.com/ItzTas/luadot/compare/f2ec7e42636ae197aaaf665a92f20076f7733638..v0.1.0-nightly.1) - 2026-08-18
#### Initial
- project initialized - ([57821a5](https://github.com/ItzTas/luadot/commit/57821a558d6d9389daefa2775c163815a672e0f7)) - Tales
#### Features
- (**alt**) add read, exists, glob and json for building files from fragments - ([f54e296](https://github.com/ItzTas/luadot/commit/f54e2961ebac5f99f70b24bbcc154b0928eb14ee)) - Tales
- (**cli**) add luadot diff for comparing managed files - ([9f1f0df](https://github.com/ItzTas/luadot/commit/9f1f0dfb404eace435180f50add433578aaee34e)) - Tales
- (**cli**) add luadot new for creating an empty template - ([4987bd4](https://github.com/ItzTas/luadot/commit/4987bd4a800b43d33027ce0bf0ecf1f8231cdf9d)) - Tales
- (**commands**) add alt, bootstrap, cd, class, completions, config, exec, restore, rm and status commands - ([be621a5](https://github.com/ItzTas/luadot/commit/be621a5f88d1d088c608e39a38b0d3a577ef8e76)) - Tales
- (**commands**) enhance add command with directory support - ([967c83f](https://github.com/ItzTas/luadot/commit/967c83f8b575eecbc8125f72ddb8806fe891eccc)) - Tales
- (**commands**) add sync command - ([bc2cfa0](https://github.com/ItzTas/luadot/commit/bc2cfa0199bbabb94745f0f2cca6b0678714e86d)) - Tales
- (**commands**) add edit command - ([8623649](https://github.com/ItzTas/luadot/commit/86236496a2f0861156bd61bb89ec33dda4817eb4)) - Tales
- (**crypt**) manage encrypted files through add, apply, edit and rm - ([533363f](https://github.com/ItzTas/luadot/commit/533363f6e7bdf2c397cf1cf52d0f2735be32bbfe)) - Tales
- (**embed**) reject nil output and unrecognized tags, slurp whitespace fully - ([bf67653](https://github.com/ItzTas/luadot/commit/bf6765367cc5a0115d433f423ce7b73fbcda26b3)) - Tales
- (**files**) manage system files under root/ - ([1091474](https://github.com/ItzTas/luadot/commit/10914745cae9f4493811b5a870a3933280124343)) - Tales
- (**files**) add sync module for syncing repository files to system - ([4ed9abe](https://github.com/ItzTas/luadot/commit/4ed9abe1f1dc92a67dabec3dc20122d8bbe6b72b)) - Tales
- (**lua**) render embedded templates - ([a7f6b43](https://github.com/ItzTas/luadot/commit/a7f6b43e9b046ca92cfb1fb4b103fa53fcde8d9c)) - Tales
- (**lua**) add ld.opt.backup and back up files before overwriting - ([4c1ef82](https://github.com/ItzTas/luadot/commit/4c1ef8244112393fbeff335feb9e648beb15c0d8)) - Tales
- (**lua**) add the ld Lua configuration interface - ([9afb0b3](https://github.com/ItzTas/luadot/commit/9afb0b362702394eb2e63044f3c8ed74062ffb78)) - Tales
- ![BREAKING](https://img.shields.io/badge/BREAKING-red) (**rules**) replace ld.git for pattern matching with ld.rules, add regex - ([c499683](https://github.com/ItzTas/luadot/commit/c499683bfe0edfdb567c1dcf145f3767f3f6f739)) - Tales
- (**rules**) let ld.rules take a single rule without wrapping it in a list - ([9598f98](https://github.com/ItzTas/luadot/commit/9598f98fe2d0f98773851b7a04712b921324ef31)) - Tales
- add passphrase encryption and rekey command - ([1357ea6](https://github.com/ItzTas/luadot/commit/1357ea6253e522ec48777a4129a73415772e6674)) - Tales
- fill functional gaps across core modules - ([4564588](https://github.com/ItzTas/luadot/commit/456458803b5f8bf5955776c1318a10c5c2a7059b)) - Tales
- run a shell command when apply or alt changes a file, set its mode - ([504d973](https://github.com/ItzTas/luadot/commit/504d97301d140253f2bff22a74b8b1d19a8418e9)) - Tales
- back up more of what luadot destroys, and let it be tuned - ([41fc8c2](https://github.com/ItzTas/luadot/commit/41fc8c244bf42af7156867b1223aa36e79060227)) - Tales
- let the managed repository's location be chosen or found - ([87b2dd1](https://github.com/ItzTas/luadot/commit/87b2dd1c966f6129b30daf92d98b2c0aba950e4b)) - Tales
- add push command - ([01267f7](https://github.com/ItzTas/luadot/commit/01267f776418a7cc4f963db79a6e505b2eaf8403)) - Tales
- add files into the repository via hard link - ([c5f8b58](https://github.com/ItzTas/luadot/commit/c5f8b58bc33f3287c8b723dd7a2a38513a0981ae)) - Tales
#### Bug Fixes
- (**add**) refuse a file a template already produces - ([6dbd292](https://github.com/ItzTas/luadot/commit/6dbd292a439e3081007912b481abe86cb82b6d93)) - Tales
- (**cli**) align the age column of luadot restore --list - ([819c6ef](https://github.com/ItzTas/luadot/commit/819c6eff13a42767d00db0e28dc952d7a1c2f89f)) - Tales
- (**config**) rename the configuration file from ld.lua to config.lua - ([c0facc1](https://github.com/ItzTas/luadot/commit/c0facc11f4dfc880d1acd4df1dac942aff4097cc)) - Tales
- (**diff**) diff the files the templates produce behind --templates - ([8dbf469](https://github.com/ItzTas/luadot/commit/8dbf4697149575e3bdf34df54ac71109712a3674)) - Tales
- (**edit**) open the script of the template that produces the file - ([dcec5f3](https://github.com/ItzTas/luadot/commit/dcec5f32ad6855b76c066404605bb62c1723113c)) - Tales
- (**rm**) take a template out whole and keep the file it produced - ([96f8889](https://github.com/ItzTas/luadot/commit/96f8889c866a642ea01f4a759618418e3e9831d1)) - Tales
- (**status**) report the files the templates produce behind --templates - ([8eb1cc3](https://github.com/ItzTas/luadot/commit/8eb1cc3025d52d5720fdc93f61225a1a4ec4fa3b)) - Tales
- (**utils**) reach a template through the path it produces - ([048bf89](https://github.com/ItzTas/luadot/commit/048bf89e333f220539a6914a11d639ff494fd6f0)) - Tales
- i think it works - ([8822a6c](https://github.com/ItzTas/luadot/commit/8822a6c90105dd58cd804c7bc9d5cbd847200c9b)) - Tales
#### Documentation
- (**claude**) forbid comments in the codebase - ([d277da0](https://github.com/ItzTas/luadot/commit/d277da0d2a75bd36e3403ac1714bca4a6bc7b82f)) - Tales
- (**embed**) note that a <%# comment's own newline survives it - ([b527fca](https://github.com/ItzTas/luadot/commit/b527fcafa622f7a0eb969214287065c6af710ad9)) - Tales
- (**readme**) document commands and the ld.lua configuration - ([52ed87c](https://github.com/ItzTas/luadot/commit/52ed87cc9b65fc96754293e2533e48067a154df6)) - Tales
- describe what the commands do with a template - ([6eb3592](https://github.com/ItzTas/luadot/commit/6eb35920a77f71faac53991c6bb6761fc8398784)) - Tales
- document encrypted files - ([6823b62](https://github.com/ItzTas/luadot/commit/6823b6240a8c2966b0960c388606e89943366a56)) - Tales
- document the home/ and root/ layout and system files - ([7ce8020](https://github.com/ItzTas/luadot/commit/7ce8020353262a7801754c7f54300cc1ef244ae8)) - Tales
- add design notes for embedded templates - ([fc0b01c](https://github.com/ItzTas/luadot/commit/fc0b01c4fe0be02d88278bd3a28bb0942c0024a4)) - Tales
- document embedded templates - ([a97b27d](https://github.com/ItzTas/luadot/commit/a97b27d4b6c8242b001a7bf4f7bf9da9dbac8327)) - Tales
- add roadmap for templates and encrypted files - ([56fc0d7](https://github.com/ItzTas/luadot/commit/56fc0d756540d10ab7b34effc0b41a80b7e9f489)) - Tales
- document commands and architecture in CLAUDE.md - ([d7d60b3](https://github.com/ItzTas/luadot/commit/d7d60b3b47bc7aec5417cf3f815ad2cffbd0efc0)) - Tales
- add project CLAUDE.md with language and change guidelines - ([a0132ba](https://github.com/ItzTas/luadot/commit/a0132ba42223798679b14a2239e8a06c2017a593)) - Tales
#### Tests
- (**alt**) confirm ld.alt.expand can call itself for partials - ([8b7eff2](https://github.com/ItzTas/luadot/commit/8b7eff2460d297c5744b0348c71234131d74a7ef)) - Tales
- (**cli**) drive both template forms end to end - ([35828b1](https://github.com/ItzTas/luadot/commit/35828b16d0b7437dfbfc8e73c7d7b1bf5163ed10)) - Tales
- (**cli**) drive the binary end to end through assert_cmd - ([7782260](https://github.com/ItzTas/luadot/commit/77822608fc05c044e488f6e8ae26c2d545f6aa68)) - Tales
- (**files**) illustrate the unreadable prediction with a system path - ([f7e45dd](https://github.com/ItzTas/luadot/commit/f7e45dd2ad2bbc62c26a13282df3afff13a038d5)) - Tales
- test - ([7892219](https://github.com/ItzTas/luadot/commit/78922198d5e2d52c55544b07f0dc2f81fa9fb72f)) - Tales
- test - ([0a74cea](https://github.com/ItzTas/luadot/commit/0a74ceacade5de3fe80fdd07f75ad0bdc7972f76)) - Tales
- test - ([0ea9ad4](https://github.com/ItzTas/luadot/commit/0ea9ad4458ca3b1bd01f0086af2b3d73e4f90409)) - Tales
#### Continuous Integration
- (**github**) build, lint and test on push - ([ab545a1](https://github.com/ItzTas/luadot/commit/ab545a1c63214573740b46e43c6a2884ae5a4590)) - Tales
- (**gitlab**) build, lint and test on push - ([7f05187](https://github.com/ItzTas/luadot/commit/7f051875a9e912d6c1036155e4933201762bbb9f)) - Tales
- (**gitlab**) mirror to gitlab - ([20ba517](https://github.com/ItzTas/luadot/commit/20ba517c49c716a66f8cb8a9bd0e4476e6ff25ce)) - Tales
- workflow now uses --force instead of --mirror - ([da63c52](https://github.com/ItzTas/luadot/commit/da63c52764b8e8b7247a10f2dfc5fc23ef295fac)) - Tales
- gitlab workflow - ([f8aed2c](https://github.com/ItzTas/luadot/commit/f8aed2cabb1cfeb9eb7b078a47338d1f560064d6)) - Tales
- does not work for now - ([536e348](https://github.com/ItzTas/luadot/commit/536e3481ee1ddb5abe1cc1d3cc5da82a98cc20a0)) - Tales
- test - ([c6ee783](https://github.com/ItzTas/luadot/commit/c6ee7836f5f250ed215dae89070d8bba15570775)) - Tales
- test - ([9ea5c07](https://github.com/ItzTas/luadot/commit/9ea5c071d9f2aa398e7f09f0c626861b23a10171)) - Tales
#### Refactoring
- (**cli**) collect the managed files in one place - ([32821a5](https://github.com/ItzTas/luadot/commit/32821a5681fcd33862e60a877b7184b7ff10f50c)) - Tales
- (**cli**) share the repository preamble across the commands - ([eadc4f3](https://github.com/ItzTas/luadot/commit/eadc4f34d8a81a99a7fc7b4d288cd7b68439b9c9)) - Tales
- (**cli**) share the run state between apply and alt - ([c758d38](https://github.com/ItzTas/luadot/commit/c758d38c23898ef96a06ad1840dcfe9a6e647293)) - Tales
- (**commands**) rewire add, apply, clone, edit, git and push to clap - ([7a3075d](https://github.com/ItzTas/luadot/commit/7a3075dea55ea12b89032c6eb34d92374b3359f0)) - Tales
- (**commands**) rename the sync command to apply - ([d01b487](https://github.com/ItzTas/luadot/commit/d01b48745f1aad3bc0eb7eff23561f0002f742a5)) - Tales
- (**files**) share how a source entry's link target is read - ([9151c5b](https://github.com/ItzTas/luadot/commit/9151c5b329fe855f64065b46a4378e560f46378a)) - Tales
- (**files**) share the filesystem primitives with crypt - ([199ece3](https://github.com/ItzTas/luadot/commit/199ece3adc3a59b1d35e64a5d7e9bcc2b797ae73)) - Tales
- (**lua**) share how the setup group finds and runs its scripts - ([5fe08fe](https://github.com/ItzTas/luadot/commit/5fe08fe227a76bd0e14bdfbbb4f03430ffbe5dd7)) - Tales
- (**lua**) name the link mode and conflict policy lookups - ([a4b84f1](https://github.com/ItzTas/luadot/commit/a4b84f1483c70d416e5b3c174e0f4c7108d39e51)) - Tales
- (**lua**) share the alt failure message and file read - ([24b37e3](https://github.com/ItzTas/luadot/commit/24b37e391ba171517a8ba465c6d821b3ad290671)) - Tales
- (**lua**) share the option table wiring between opt and crypt - ([ab74ae5](https://github.com/ItzTas/luadot/commit/ab74ae5560b3d3692abcfa292bead7806695348c)) - Tales
- (**lua**) build the opt and crypt tables from their setter lists - ([6933a28](https://github.com/ItzTas/luadot/commit/6933a2821fb023b2f13859c5ab6d845e9e04b55a)) - Tales
- (**lua**) share the ld value coercions across opt and crypt - ([83b501e](https://github.com/ItzTas/luadot/commit/83b501e862f2594bc3f9919d7d6a6842c77e434f)) - Tales
- ![BREAKING](https://img.shields.io/badge/BREAKING-red) (**paths**) nest the mirrors under home/ and root/ - ([fb943eb](https://github.com/ItzTas/luadot/commit/fb943eb18e1fb427daafc4f6347f0c8e0cfa6251)) - Tales
- (**state**) encapsulate config behind accessor methods - ([cb2ca40](https://github.com/ItzTas/luadot/commit/cb2ca40b37e77f978b799198ef47023449405249)) - Tales
- (**utils**) share how a template resolves and how its files compare - ([8d7da5d](https://github.com/ItzTas/luadot/commit/8d7da5d6f8da6e5951a137dd4a6ce63f23ff451e)) - Tales
- (**utils**) move backup, hook, preview and prompt out of utils - ([b74d7ab](https://github.com/ItzTas/luadot/commit/b74d7ab6dc4251ee3072947e2d1cf7868308e143)) - Tales
- (**utils**) extract shared home-relative path expansion - ([bcb30aa](https://github.com/ItzTas/luadot/commit/bcb30aa9910ab7d372b16e4c563d32bbeb47e756)) - Tales
- rename clone functions and modules - ([3488702](https://github.com/ItzTas/luadot/commit/3488702db906aec0017dc1e4a9051db8e231eda9)) - Tales
#### Miscellaneous Chores
- (**bench**) measure the embedded templates - ([bc1b2c0](https://github.com/ItzTas/luadot/commit/bc1b2c00d32407f8c0f853477d2baa5737169ace)) - Tales
- (**cli**) extract shared CLI constants into a dedicated module - ([7729ad0](https://github.com/ItzTas/luadot/commit/7729ad0af2978efb96f6e751c274d1427dd32ed2)) - Tales
- (**commands**) remove redundant comment from push command - ([8f5fcaf](https://github.com/ItzTas/luadot/commit/8f5fcaf798d2089086d2bbf7bea6141b0e319a2e)) - Tales
- (**crypt**) drop an unused import - ([848cd0f](https://github.com/ItzTas/luadot/commit/848cd0f0217c88330326115544ceb8bbf2b7ac51)) - Tales
- (**crypt**) add age and gpg primitives for encrypted files - ([34697ad](https://github.com/ItzTas/luadot/commit/34697ad0290d4c014f7a540d6ce2302c0ebb665c)) - Tales
- (**deps**) add assert_cmd and predicates - ([797a3e7](https://github.com/ItzTas/luadot/commit/797a3e711f8d50d70be3017e112c4a8c90678aa5)) - Tales
- (**deps**) add benchmarking, CLI parsing and logging dependencies - ([69f4cc8](https://github.com/ItzTas/luadot/commit/69f4cc8da90f17f21d9bfb182c82de7d027a12a8)) - Tales
- (**deps**) add anstream, anstyle and glob - ([1a9a79a](https://github.com/ItzTas/luadot/commit/1a9a79a4abcce599e8eabff436469b149074f2a5)) - Tales
- (**files**) add a copy link mode - ([4094c4d](https://github.com/ItzTas/luadot/commit/4094c4d7942fe38c4a347a43c8454e512be76863)) - Tales
- (**files**) add predict helper for the outcome of a write - ([c8581b3](https://github.com/ItzTas/luadot/commit/c8581b3c4b339fc43565f250cb82d17095c7f478)) - Tales
- (**files**) add status, template, walk and write helpers - ([d7d9a56](https://github.com/ItzTas/luadot/commit/d7d9a561306f0a16f978bc66cc399e47acdd4abf)) - Tales
- (**hooks**) run checks before push instead of every commit - ([5a69385](https://github.com/ItzTas/luadot/commit/5a6938554f514e99724053fd56d9dd984a16a79d)) - Tales
- (**lua**) add the ld.crypt configuration group - ([76c0a65](https://github.com/ItzTas/luadot/commit/76c0a65c96a99148db170c34dde671ff2c368e12)) - Tales
- (**lua**) move opt's Setter type and SETTERS table into constants.rs - ([9284afa](https://github.com/ItzTas/luadot/commit/9284afa1e38b73b5487061e8ec6b44afb77c8b4b)) - Tales
- (**lua**) add the embedded template compiler - ([291d60d](https://github.com/ItzTas/luadot/commit/291d60dc36e552083b402a9f4f299aab0b3b7a4b)) - Tales
- (**output**) add colored status output module - ([211039b](https://github.com/ItzTas/luadot/commit/211039b0e7e938078b6f4da9b48440080e72d861)) - Tales
- (**rules**) carry an encrypt flag on rules - ([39039da](https://github.com/ItzTas/luadot/commit/39039da309f9d56cfe295f59635754a5e783ff44)) - Tales
- (**rules**) carry a mode and an owner on rules - ([99d84a3](https://github.com/ItzTas/luadot/commit/99d84a3200e4678042c4e1946895b58dafc5dd00)) - Tales
- (**skills**) update design-system for clap and integration tests - ([31b28e3](https://github.com/ItzTas/luadot/commit/31b28e3952e9eab09366fb40266852d9ab48c254)) - Tales
- (**skills**) document benches and the library crate in design-system - ([715c8c3](https://github.com/ItzTas/luadot/commit/715c8c3d868dbd74eba264f2968ebd23238a0e2d)) - Tales
- (**skills**) add design-system skill guide - ([c3473b3](https://github.com/ItzTas/luadot/commit/c3473b39b4004fbe0183f94c2ff4c4bdcd7030e5)) - Tales
- (**state**) store the machine's host classes - ([ec18ccc](https://github.com/ItzTas/luadot/commit/ec18ccce556b510cab87f8606de111b501fc0ba3)) - Tales
- (**state**) add host classes - ([f2fc15f](https://github.com/ItzTas/luadot/commit/f2fc15fd5008264fb98920c5308c0a71824df718)) - Tales
- (**utils**) add editor, prompt, repo and classes helpers - ([bb6a216](https://github.com/ItzTas/luadot/commit/bb6a21631f9c4555df67ae06ead13cd67a8040a2)) - Tales
- (**version**) v0.1.0-nightly.1 [skip ci] - ([8ae3260](https://github.com/ItzTas/luadot/commit/8ae32600854c0465d82e609b6d2e8317eb053108)) - github push repo
- update project metadata and tooling config - ([56cc5e2](https://github.com/ItzTas/luadot/commit/56cc5e25876c093fd33530269c0ba8eaf21d7bd9)) - Tales
- add AUR packaging support - ([97e43e1](https://github.com/ItzTas/luadot/commit/97e43e1317cb49edd8b5e9a6fb929cbb78230ae0)) - Tales
- run cargo fmt - ([a716781](https://github.com/ItzTas/luadot/commit/a71678128d84bd5c75220359e99911cd79ca32f9)) - Tales
- run cargo fmt - ([c8cbf23](https://github.com/ItzTas/luadot/commit/c8cbf23244187bf6e5de143871ccd1b3a54ef651)) - Tales
- also run cargo clippy in pre-commit hook - ([5f7f529](https://github.com/ItzTas/luadot/commit/5f7f5290acf9a0923593a83e84e0a73dab1ab54b)) - Tales
- run cargo test in pre-commit hook - ([b590793](https://github.com/ItzTas/luadot/commit/b5907934f2ca759331c7856672809060f7c2f134)) - Tales
- add serde tests for state type - ([de4a676](https://github.com/ItzTas/luadot/commit/de4a6763c6085cf44fe8476ac10bf7acb59fc192)) - Tales
- make state store testable and add tests - ([6b27af5](https://github.com/ItzTas/luadot/commit/6b27af5b0560d5ba2a372ba34240495662241383)) - Tales
- make data_dir testable and add tests - ([dc7d2a5](https://github.com/ItzTas/luadot/commit/dc7d2a55997dd9c13066790640834385744d9c0a)) - Tales
- add test for lua runtime - ([6d41abd](https://github.com/ItzTas/luadot/commit/6d41abd1fb4933ee032334b7a7f7b1dce50ce66d)) - Tales
- add tests for cli command dispatch - ([b766800](https://github.com/ItzTas/luadot/commit/b766800941ab35c23071eafef37bbe24db8fbcfd)) - Tales
- add test for git clone_repo - ([905fa0e](https://github.com/ItzTas/luadot/commit/905fa0e574fd1ac26ceecb7e96cecb8a2880f492)) - Tales
- rename git clone module to clone_repo - ([8b08829](https://github.com/ItzTas/luadot/commit/8b0882911e1bffc83d1964c82e8d1840badddcd5)) - Tales
- claude md modified - ([1a2d0f1](https://github.com/ItzTas/luadot/commit/1a2d0f14604cb65a110bdb791226180557c366ea)) - Tales
- extract cli dispatch into a run module - ([cde3261](https://github.com/ItzTas/luadot/commit/cde32615468a92e238b216a80c39393d0b5de84e)) - Tales
- add cli command registry and clone command - ([1832de9](https://github.com/ItzTas/luadot/commit/1832de9e77fe60f412c7a58b5b9d1b388c894f09)) - Tales
- add gix-based repository clone helper - ([6f1b019](https://github.com/ItzTas/luadot/commit/6f1b01916a16b77ae1aeda8b76a14f95bc7831c5)) - Tales
- add persistent json state store - ([fcd9914](https://github.com/ItzTas/luadot/commit/fcd9914259ce89c0b8c19be1b16f771405a6d24e)) - Tales
- add data_dir path helper - ([32e531f](https://github.com/ItzTas/luadot/commit/32e531f5900ec32b64c96a40011a1ffffb958e51)) - Tales
- remove lua template demo and host api - ([7a9c542](https://github.com/ItzTas/luadot/commit/7a9c54271d9bf9651d0e3f408ece958bfc2eeb50)) - Tales
- add gix, serde and serde_json dependencies - ([c29417a](https://github.com/ItzTas/luadot/commit/c29417acb2502f34cc5a1bff7294ce5d03e2a808)) - Tales
- split GitLab mirror into clone and retried push - ([fc4c162](https://github.com/ItzTas/luadot/commit/fc4c162c5f122b210b40b806cf2c13b21b443b26)) - Tales
- scaffold base modules for cli, files, git and utils - ([f95daea](https://github.com/ItzTas/luadot/commit/f95daeae534685eb5c65b3c6b3bd5c39ae93211e)) - Tales
- test - ([2336f2b](https://github.com/ItzTas/luadot/commit/2336f2bed286e848f9b3bbd84c3a88649db722de)) - Tales
- this is from gitlab again 2 - ([dc571bc](https://github.com/ItzTas/luadot/commit/dc571bcab2e68686fc297d7fdee351d77e382738)) - Tales
- this is from gitlab again - ([f141a60](https://github.com/ItzTas/luadot/commit/f141a6072e24a6fe31296bc148ffcb0b95a3128d)) - Tales
- this is from gitlab - ([b1ab1a9](https://github.com/ItzTas/luadot/commit/b1ab1a9827d3170fdfe29d864bdf8dfed207d5e6)) - Tales
- test - ([9e8054e](https://github.com/ItzTas/luadot/commit/9e8054ec8beed5228508a0ba1a5b95f768d20846)) - Tales
- test - ([5a27b18](https://github.com/ItzTas/luadot/commit/5a27b185a9142ed79720ac58562481f7785463e0)) - Tales
- diagnose gitlab removed - ([87981ad](https://github.com/ItzTas/luadot/commit/87981ad002e6b76d142fb8bebebb45ac8af6e73c)) - Tales
- test - ([69ef221](https://github.com/ItzTas/luadot/commit/69ef22179c51a3904689f0716b2833fb6bbaff74)) - Tales
- test - ([08ad95a](https://github.com/ItzTas/luadot/commit/08ad95af8500fb25d43414f6788d29cc2fe83eda)) - Tales
- test - ([89c6a1c](https://github.com/ItzTas/luadot/commit/89c6a1c6f4fa26e130f4a5a601a16792e4bb7f4d)) - Tales
- test - ([0445a59](https://github.com/ItzTas/luadot/commit/0445a59b655ef14f36c083b571c3b4b2cbab156b)) - Tales
- test - ([0d4a398](https://github.com/ItzTas/luadot/commit/0d4a39877f0839030f8eadb503fc52ebf9485aae)) - Tales
- lua added - ([448b7c8](https://github.com/ItzTas/luadot/commit/448b7c87b28c8e497fb411c8f1f1bd360b794365)) - Tales
- gitignore updated - ([45be0ab](https://github.com/ItzTas/luadot/commit/45be0ab68d03e14eb4620ae58bc6fad7c88f78a7)) - Tales
- commit linting - ([8b41488](https://github.com/ItzTas/luadot/commit/8b41488af0515a2350852bea3f4f8eef53551c04)) - Tales

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).