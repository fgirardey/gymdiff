# Design tokens

Single reference for the visual foundations. **Mirrored 1:1** in `crates/gymdiff-app/ui/theme.slint` (global `Theme`) and in the `Foundations` page of `design/gymdiff.pen`.

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
| `danger` | `#f85149` | Deaths, losses, disconnected, errors |

`accent` and `success` share a value on purpose (green is the brand colour), but keep the roles separate in code.

## Spacing
| Token | Value |
|---|---|
| `space-xs` | 4px |
| `space-s` | 8px |
| `space-m` | 16px |
| `space-l` | 24px |

## Radius
| Token | Value |
|---|---|
| `radius-m` | 8px |

## Typography
| Token | Size | Usage |
|---|---|---|
| `font-size-body` | 14px | Body text |
| `font-size-title` | 20px | Titles |
| `font-size-display` | 40px | Big numbers (clock, stats) |

Font families (Pencil variables `font-display` and `font-body`):

| Role | Family | Usage | License |
|---|---|---|---|
| Display | Barlow Condensed (ExtraBold/Bold Italic) | Wordmark, titles, big numbers | SIL Open Font License 1.1 |
| Body | Inter | Body text, labels | SIL Open Font License 1.1 |

The fonts are not bundled in the app yet: until they are, `theme.slint` has no font-family tokens and Slint uses the system font. Bundling means adding the `.ttf` files and their `OFL.txt` to the repository and importing them from `theme.slint`.

## Naming rules
- kebab-case, same name everywhere (Pencil variable, this table, `Theme.<name>` in Slint).
- Name by role (`text-secondary`), not by look (`grey-400`).
