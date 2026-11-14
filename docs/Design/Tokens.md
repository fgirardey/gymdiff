# Design tokens

Single reference for the visual foundations. **Mirrored 1:1** in `crates/cooldown-app/ui/theme.slint` (global `Theme`) and in the `Foundations` page of `design/cooldown.pen`.

When a value changes: update Pencil, this note and `theme.slint` in the same commit.

Colours come from the brand direction chosen in Pencil (see [[Brand]]): a K/DA reading where green is what you gained and red what you gave away.

## Colors
| Token | Value | Usage |
|---|---|---|
| `background` | `#0d1117` | Window background |
| `surface` | `#161b22` | Cards, panels |
| `border` | `#30363d` | Dividers, outlines |
| `text-primary` | `#e6edf3` | Main text |
| `text-secondary` | `#8b949e` | Secondary text, labels |
| `accent` | `#3fb950` | Brand accent, highlights |
| `success` | `#3fb950` | Kills, gains, connected |
| `danger` | `#f85149` | Deaths, losses, disconnected, errors, destructive actions (Uninstall) |
| `warning` | `#d29922` | Partial workouts, sessions with some workouts missed, a game's cap applied |
| `info` | `#58a6ff` | Wins (result, stripe and tint of a History row, "W" chip) |
| `scrim` | `#010409c7` | Dims the window behind a dialog (workout prompt, uninstall confirmation) |

`accent` and `success` share a value on purpose (green is the brand colour), but keep the roles separate in code.

## Spacing
| Token | Value |
|---|---|
| `space-xs` | 4px |
| `space-s` | 8px |
| `space-m` | 16px |
| `space-l` | 24px |
| `space-xl` | 32px |

## Radius
| Token | Value |
|---|---|
| `radius-chip` | 4px (session header chips) |
| `radius-s` | 6px (view switch segments, row chevron) |
| `radius-m` | 8px |
| `radius-dialog` | 16px |
| `radius-panel` | 10px (session panel, champion portrait) |

## Session panel
Tints of a session panel ([[History#Session summary]]): the session's colour (`text-secondary` for a session of games from before cooldown.lol) at these opacities. They are Pencil variables, but the `Session / *` components still use literal values.

| Token | Value | Usage |
|---|---|---|
| `tint-header` | 0.12 | Summary header background |
| `tint-line` | 0.4 | Line under the summary header |
| `tint-games` | 0.04 | Games area background |
| `muted-opacity` | 0.75 | "No workout" card |

## Game row
Tints of a History row in the Games view ([[History#Game rows]]): the result's colour fades from the stripe on the left, at these opacities at 0 % and 28 % of the row's width, and is gone at 60 %.

| Token | Value | Usage |
|---|---|---|
| `tint-row` | 0.22 | Win or loss tint, at the stripe |
| `tint-row-mid` | 0.08 | Win or loss tint, at 28 % |
| `tint-row-muted` | 0.15 | `text-secondary` tint of a game without workout, at the stripe |
| `tint-row-muted-mid` | 0.05 | Same, at 28 % |

## Interaction
Hover, pressed, focus, disabled and selected states. Buttons, icon buttons and text links are Pencil components in `Controls library`, with one reusable instance per state (`Button / Primary / Hover`, …) and a **State rules** table (trigger, visual change, tokens, motion, cursor); switches and workout cards are in the `Interaction states` section of the `Foundations` page. Every interactive control adds a **state layer** in its content colour (Material), filled and outlined buttons also **shrink** while pressed (iOS), and every change eases out.

| Token | Value | Usage |
|---|---|---|
| `state-opacity-hover` | 0.08 | State layer opacity while hovered |
| `state-opacity-pressed` | 0.12 | State layer opacity while pressed |
| `press-scale` | 0.97 | Scale of filled and outlined buttons while pressed |
| `disabled-opacity` | 0.4 | Opacity of a disabled control (e.g. **Check now** while a check runs) |
| `motion-fast` | 100ms | Hover and press transitions (ease-out) |
| `motion-medium` | 200ms | Switch toggle, card expand and collapse (ease-out) |
| `focus-ring` | `#58a6ff` | Keyboard focus ring (Tab only, never on click) |
| `focus-ring-width` | 2px | Focus ring thickness, drawn outside the control and following its radius; replaces the border of outlined buttons |

The state layer colour is the control's content colour: `background` on `accent` buttons and on the "on" switch track, `text-primary` everywhere else. Text links have no layer: they turn `text-primary` on hover and drop to 60% opacity while pressed. In Pencil the durations are plain numbers in milliseconds.

In Slint, `components/state-layer.slint` (`StateLayer`) draws the layer: every interactive control lays one over its background, fed by its `TouchArea`'s `has-hover` and `pressed`.

## Typography
| Token | Size | Usage |
|---|---|---|
| `font-size-micro` | 10px | A game's cap ("max 30") |
| `font-size-small` | 11px | Captions of the formula in a History row's details |
| `font-size-caption` | 12px | Small labels (top bar status) |
| `font-size-label` | 13px | Workout status, view switch, exercise names |
| `font-size-body` | 14px | Body text |
| `font-size-wordmark` | 18px | "COOLDOWN.LOL" wordmark in the top bar |
| `font-size-title` | 20px | Titles, stat values of History rows |
| `font-size-equation` | 22px | Numbers and operators of the formula in a History row's details |
| `font-size-headline` | 32px | Welcome headline |
| `font-size-display` | 40px | Big numbers (clock, stats) |
| `font-size-hero` | 96px | The one number to read from a distance (workout step count, plank countdown) |

Font families (Pencil variables `font-display` and `font-body`):

| Role | Family | Usage | License |
|---|---|---|---|
| Display | Barlow Condensed (ExtraBold/Bold Italic) | Wordmark, titles, big numbers | SIL Open Font License 1.1 |
| Body | Inter | Body text, labels | SIL Open Font License 1.1 |

The fonts are bundled in `crates/cooldown-app/ui/fonts/` with their `OFL.txt` (Barlow Condensed ExtraBold Italic, Bold Italic, ExtraBold and SemiBold, the weights the design uses; Inter as a variable font) and imported from `theme.slint`, which mirrors the two families as `font-display` and `font-body`. The windows set `default-font-family` to `font-body`, and every `Text` sets its family explicitly so previews render like the app: `font-display` for the wordmark, titles, results and big numbers, `font-body` for the rest.

## Naming rules
- kebab-case, same name everywhere (Pencil variable, this table, `Theme.<name>` in Slint).
- Name by role (`text-secondary`), not by look (`grey-400`).
