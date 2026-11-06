# Brand

GymDiff's identity. The reference is the `Brand / Logo` frame in `design/gymdiff.pen`; colours and fonts are in [[Tokens]].

## Idea
The name reads two ways: **gym** (training, getting better) and **diff** (League slang for the gap between two players, as in "jungle diff"). The identity reads like a K/DA line: **green is what you gained, red what you gave away**.

## Logo: Crest
A shield (ranked-ladder, competitive) holding a green chevron up and a red chevron down.

| Asset | Pencil component | File |
|---|---|---|
| Crest (mark alone, also the app icon) | `Logo / Crest` | `design/brand/crest.svg`, `design/brand/crest-256.png` |
| Lockup (crest + wordmark) | `Logo / Lockup` | `design/brand/lockup.svg` |

- **Wordmark**: `GYMDIFF` in Barlow Condensed ExtraBold Italic; `GYM` in `text-primary` (or `background` on light), `DIFF` in `accent`.
- **App icon**: the crest on its own, no tile, transparent background. Windows `.ico` sizes: 256, 48, 32, 24, 16.
- **Mono**: single-colour versions (white or `background`) only where colour is impossible.

Re-export the files from Pencil whenever a logo component changes.

## Rules
- Clear space: at least the height of one chevron around the crest and the lockup.
- Minimum size: crest 16 px; lockup 24 px high. Below that, use the crest alone.
- Green is always up and red always down. Never swap them.
- No effects, no stretching, no busy image behind the crest.
- Never pair the logo with Riot logos or League art, and never make GymDiff look official (see [[Riot policies]]).

## History
Four directions were explored first (Delta, Forge, Training Ground, Level Up), then four sport/gaming marks on the Delta green/red palette (Dumbbell Diff, GD Strike, Crest, Versus). Crest was chosen for its legibility at 16 px and its competitive feel. The explorations were removed from `design/gymdiff.pen` to keep a single version.
