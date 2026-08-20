# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [v0.1.0-nightly.4](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.3..v0.1.0-nightly.4) - 2026-08-18

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

## [v0.1.0-nightly.3](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.2..v0.1.0-nightly.3) - 2026-08-18
#### Documentation
- (**design-system**) rule out tests another test already covers - ([5033caf](https://github.com/ItzTas/luadot/commit/5033caf5f31d2cdc1c7bbaaf8c68eb1c86bac426)) - Tales
#### Tests
- drop tests already covered by other tests - ([e38fe5d](https://github.com/ItzTas/luadot/commit/e38fe5d34b9daae88251934903207655b793748b)) - Tales

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

## [v0.1.0-nightly.2](https://github.com/ItzTas/luadot/compare/v0.1.0-nightly.1..v0.1.0-nightly.2) - 2026-08-18
#### Miscellaneous Chores
- (**aur**) drop check() from PKGBUILD templates (#7) - ([3373052](https://github.com/ItzTas/luadot/commit/3373052ddc06aeecf8a67ef5869f31697ac05c92)) - Tales Sabini

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