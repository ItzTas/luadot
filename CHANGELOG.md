# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

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

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).