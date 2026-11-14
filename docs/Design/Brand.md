# Brand

cooldown.lol's identity. The reference is the `Brand / Logo` frame in `design/cooldown.pen`; colours and fonts are in [[Tokens]].

> The product was called GymDiff until [[ADR-0015 Rename to cooldown.lol]].

## Idea
The name reads two ways: an ability **on cooldown** in League, and the **cool-down** after a workout, the time between two games. The identity keeps the K/DA reading: **green is what you kept, red what you pay in reps**.

## Logo: Mark
An hourglass in a rounded tile with a green outline, the shape of a League spell icon on cooldown. Green sand on top, red sand at the bottom.

| Asset | Pencil component | File |
|---|---|---|
| Mark (also the app icon) | `Logo / Mark` | `design/brand/crest.svg`, `design/brand/crest-256.png` |
| App icon | `Logo / Mark` | `design/brand/cooldown.ico` (Windows executable), `crates/cooldown-app/ui/images/app-icon.png` (window icon, 256 px) |
| Lockup (mark + wordmark) | `Logo / Lockup` | `design/brand/lockup.svg` |

`crest.*` keep the name of the first mark.

- **Wordmark**: `COOLDOWN.LOL` in Barlow Condensed ExtraBold Italic; `COOLDOWN` in `text-primary` (or `background` on light), `.LOL` in `accent`.
- **Name in copy**: `cooldown.lol`, lowercase, in sentences; `COOLDOWN.LOL` in uppercase titles.
- **App icon**: the mark on its own; the tile is part of it, no extra background. Windows `.ico` sizes: 256, 48, 32, 24, 16.
- **Mono**: single-colour versions (white or `background`) only where colour is impossible: tile outline and glass in the colour, sand cut out.

Re-export the files from Pencil whenever a logo component changes. The `.ico` holds one PNG per size: export `Logo / Mark` (128 px) at scales 2, 0.375, 0.25, 0.1875 and 0.125, then pack the five PNGs into `cooldown.ico` (PNG-compressed entries, 32-bit). `build.rs` embeds it in `cooldown.exe` on Windows.

## Rules
- Clear space: at least a quarter of the mark's width around the mark and the lockup.
- Minimum size: mark 16 px; lockup 24 px high. Below that, use the mark alone.
- Green sand is always on top and red sand always at the bottom. Never swap them.
- Never drop the tile, add effects, stretch the mark or put it on a busy image.
- Never pair the logo with Riot logos or League art, and never make cooldown.lol look official (see [[Riot policies]]).
- The name and logo are not covered by the code license ([[ADR-0009 Open-source license]]): forks use their own.

## History
GymDiff first used a **Crest** (a ranked shield holding a green chevron up and a red chevron down), chosen over Delta, Forge, Training Ground, Level Up, Dumbbell Diff, GD Strike and Versus. For the rename to cooldown.lol, four marks were explored (ability sweep, rep-timer C, frost kettlebell, hourglass), then six hourglass variants; **Spell on cooldown** (the hourglass in a spell tile) was chosen. The explorations are on the `Rebrand / cooldown.lol exploration` board in `design/cooldown.pen`.
