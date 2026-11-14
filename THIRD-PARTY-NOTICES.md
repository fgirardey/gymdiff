# Third-party notices

cooldown.lol's own code and assets are licensed under MIT OR Apache-2.0 (see `LICENSE-MIT` and `LICENSE-APACHE`). The files below are **not** covered by that license.

## Riot Games assets
- `crates/cooldown-app/ui/images/roles/*.png`, `design/assets/roles/*.png`: League of Legends role symbols.
- `design/assets/champions/*.png`: champion portraits from Data Dragon, used in the design mock-ups only.

League of Legends and all associated properties are trademarks or registered trademarks of Riot Games, Inc. These files are used under Riot's ["Legal Jibber Jabber"](https://www.riotgames.com/en/legal) policy for free, non-commercial community projects (see `docs/Decisions/ADR-0005 Bundle the role symbols.md`). They grant no rights to reuse them elsewhere.

cooldown.lol isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties.

## cooldown.lol name and logo
- `design/brand/*`, `crates/cooldown-app/ui/images/mark.svg`, `crates/cooldown-app/ui/images/app-icon.png`: the cooldown.lol name, mark and lockup.

The code license does not grant the right to use the cooldown.lol name or logo for another product or a modified version presented as cooldown.lol. Forks must use their own name and logo.

## Fonts
- `crates/cooldown-app/ui/fonts/BarlowCondensed-*.ttf`: [Barlow Condensed](https://github.com/jpt/barlow), Copyright 2017 The Barlow Project Authors, under the SIL Open Font License 1.1 (`crates/cooldown-app/ui/fonts/BarlowCondensed-OFL.txt`). From the Google Fonts repository.
- `crates/cooldown-app/ui/fonts/Inter-VariableFont.ttf`: [Inter](https://github.com/rsms/inter), Copyright 2020 The Inter Project Authors, under the SIL Open Font License 1.1 (`crates/cooldown-app/ui/fonts/Inter-OFL.txt`). The copy Slint ships in `i-slint-common`.

Both are embedded in the app and installer executables.

## Lucide
- `crates/cooldown-app/ui/images/icons/settings.svg` and `refresh-cw.svg`, and `arrow-right`, `chevron-down`, `dumbbell`, `eye`, `flame`, `skull`, `swords` and `wheat` in the same folder (exported as outlines from `design/cooldown.pen`), from [Lucide](https://lucide.dev), under the ISC License:

```
ISC License

Copyright (c) for portions of Lucide are held by Cole Bemis 2013-2022 as part of Feather (MIT).
All other copyright (c) for Lucide are held by Lucide Contributors 2022.

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```

## Slint
The UI toolkit [Slint](https://slint.dev) is used under the Slint Royalty-free Desktop License, which requires the "Made with Slint" notice shown in the About screen (see `docs/Decisions/ADR-0001 Native UI with Slint.md`).
