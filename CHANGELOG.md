# Changelog

## 0.1.0 (2026-11-14)


### ⚠ BREAKING CHANGES

* GymDiff installs do not update to cooldown.lol and their history and settings are not carried over: uninstall GymDiff, then install cooldown.lol-alpha-Setup.exe.

### refactor

* rename GymDiff to cooldown.lol ([bf5c4d9](https://github.com/fgirardey/cooldown.lol/commit/bf5c4d9aef295ef1d9c5b4874441748103aa9dbe))


### Features

* add a read-only recorder that saves real sessions as scenarios ([cc43a6e](https://github.com/fgirardey/cooldown.lol/commit/cc43a6e55903da8b65c4f637b0d7d1ab2cfb7372))
* add an installer window that asks for the shortcuts ([d2de572](https://github.com/fgirardey/cooldown.lol/commit/d2de572ebe0962935ab65d30d1a9bd4e17d534f6))
* **app:** add a throwaway debug screen to fire test events ([d06253e](https://github.com/fgirardey/cooldown.lol/commit/d06253e17e67211ef8ec711620536870d5c8ab92))
* **app:** add the building blocks of opt-in error reports ([64e3fad](https://github.com/fgirardey/cooldown.lol/commit/64e3fadca9423efbfd9d1421c128c6613a3cc5a8))
* **app:** give GymDiff its crest as app icon ([9d0c50a](https://github.com/fgirardey/cooldown.lol/commit/9d0c50ae45de6be2cac424ddd3a4dc41e968b1e1))
* **app:** identify each build by version, build number, commit and channel ([018d2ba](https://github.com/fgirardey/cooldown.lol/commit/018d2ba6937911386bd3fde7da329201872e6681))
* **app:** install and update GymDiff with Velopack ([9012024](https://github.com/fgirardey/cooldown.lol/commit/901202460681c5c1b6d00ef7ff160c121dd23716))
* **app:** manage the Start menu and Desktop shortcuts in Settings ([2892bd2](https://github.com/fgirardey/cooldown.lol/commit/2892bd2a0f226abdc258e742c694a0ba1c2646ed))
* **app:** open no console window in Windows release builds ([80e8453](https://github.com/fgirardey/cooldown.lol/commit/80e8453b9b9d665f3659f31c995f7f529f255aa9))
* **app:** send opt-in error reports to Sentry ([5c13306](https://github.com/fgirardey/cooldown.lol/commit/5c13306ed683e72c8d81ccfd9f476ceda79a7b46))
* **app:** send Sentry issues and structured logs, one switch each ([747f346](https://github.com/fgirardey/cooldown.lol/commit/747f346e2042fb7d964e0d11804f529304ca3038))
* **app:** send symbolicated stack traces and native crash reports ([61eb012](https://github.com/fgirardey/cooldown.lol/commit/61eb012048e50cbe4b4c74c482072e60ad97aaa8))
* **app:** show the history of games from before GymDiff ([c76ee59](https://github.com/fgirardey/cooldown.lol/commit/c76ee59eeefa693f67a17749fba3f0c4ed1505b4))
* **app:** show the welcome screen on first launch, then an empty history ([f35b589](https://github.com/fgirardey/cooldown.lol/commit/f35b589454d0b49a1b78ba69d7e874ae453bb3bd))
* **app:** store preferences in a single settings.json ([d09210c](https://github.com/fgirardey/cooldown.lol/commit/d09210c0afca9b0fcb39be3aaf2291d8a3e53a83))
* **app:** store the game history in SQLite and import past games ([2688844](https://github.com/fgirardey/cooldown.lol/commit/26888442066cb361296ab06d517c2907ec843f11))
* **app:** update alpha builds to the latest build of main ([8b1aa3b](https://github.com/fgirardey/cooldown.lol/commit/8b1aa3b2501969b060266398e7b334f64836eb31))
* **app:** update from the latest release of the channel ([ac27d4d](https://github.com/fgirardey/cooldown.lol/commit/ac27d4d8d250189546d25c015fb894a878c1b859))
* **design:** add interaction states to the Pencil design ([3aead02](https://github.com/fgirardey/cooldown.lol/commit/3aead021faa31a6e58d93597fec2dcca9d0b2c94))
* **design:** add the Updates section to Settings ([eeca57c](https://github.com/fgirardey/cooldown.lol/commit/eeca57c11955067d1560453f4815f3ba75f5dbfc))
* **design:** design the installer window and shortcut settings ([0580bb8](https://github.com/fgirardey/cooldown.lol/commit/0580bb8ae81cafde025255394bee177df93697b9))
* **design:** design the startup, uninstall and installer options ([0e66f76](https://github.com/fgirardey/cooldown.lol/commit/0e66f76637fed965613ca4fe67dd6b8687142914))
* read the active summoner's past games from the client ([ca73aab](https://github.com/fgirardey/cooldown.lol/commit/ca73aabe864a22a06566ea015a16740d15a59e76))
* redesign the app for a 1280 × 800 window ([1fe9eb3](https://github.com/fgirardey/cooldown.lol/commit/1fe9eb3a393f27a36168ac305d1c1dfd8810dfd0))
* run in the notification area, start with Windows, uninstall ([e93148a](https://github.com/fgirardey/cooldown.lol/commit/e93148acff0559576d993ad970c05a00fcbf5bea))
* **ui:** add caption, wordmark, headline and space-xl tokens ([5293faf](https://github.com/fgirardey/cooldown.lol/commit/5293faf60fd1baea7c8b5d244efc0f9222962b75))
* **ui:** add champion portraits with bundled role symbols ([e2a17ca](https://github.com/fgirardey/cooldown.lol/commit/e2a17ca3dfdcd30187a2060523e2a424f8440bc9))
* **ui:** add the Crest logo, brand note and real design tokens ([d0d30a6](https://github.com/fgirardey/cooldown.lol/commit/d0d30a69fe9da81fdee32fdb484649e4997aa689))
* **ui:** add the settings screen with the error reports switches ([73c3fe3](https://github.com/fgirardey/cooldown.lol/commit/73c3fe31b13f552c2e80cd8d43336276b442b2a8))
* **ui:** add warning and hero font size tokens ([a9fe473](https://github.com/fgirardey/cooldown.lol/commit/a9fe4738c1adf05014dd830c37a7ec9f6edaec02))


### Bug fixes

* **app:** log watcher state changes instead of every poll ([dfdd296](https://github.com/fgirardey/cooldown.lol/commit/dfdd296f4ee41d45aaecbe0f2e77b57ae7a93488))
* **ci:** import the Sentry upload script through a file URL ([629109e](https://github.com/fgirardey/cooldown.lol/commit/629109e71f9388f10fd23d8b907592283cc554b7))
* **ci:** let the pr-builds job check out the private repository ([967676a](https://github.com/fgirardey/cooldown.lol/commit/967676aea76fbf505e3153cb6fb6c6ef5316d398))
* **ci:** make v0.1.0 the first release ([77895a7](https://github.com/fgirardey/cooldown.lol/commit/77895a7aeadc0766faf90e79ed11f4b2dd1eebac))
* open Velopack's launcher under its real name, cooldown.lol.exe ([#13](https://github.com/fgirardey/cooldown.lol/issues/13)) ([e480b3b](https://github.com/fgirardey/cooldown.lol/commit/e480b3bf13201dd5e9361f152562c1634b8c08f5))
* pass CI on the latest stable toolchain and on Windows ([dc86551](https://github.com/fgirardey/cooldown.lol/commit/dc86551189cc5eb34914ce1b9fea2b801676719e))
* **tooling:** show preview paths relative to ui in the screenshots table ([938876b](https://github.com/fgirardey/cooldown.lol/commit/938876ba304f3102fed2d3ac089dfab368f6db11))
