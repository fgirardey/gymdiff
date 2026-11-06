# Design tokens

Single reference for the visual foundations. **Mirrored 1:1** in `crates/gymdiff-app/ui/theme.slint` (global `Theme`) and in the `Foundations` page of `design/gymdiff.pen`.

When a value changes: update Pencil, this note and `theme.slint` in the same commit.

> The values below are **placeholders** chosen for the skeleton. Replace them once the visual direction is set in Pencil.

## Colors
| Token | Value | Usage |
|---|---|---|
| `background` | `#0f1115` | Window background |
| `surface` | `#181b22` | Cards, panels |
| `text-primary` | `#e8e6e3` | Main text |
| `text-secondary` | `#9aa0a6` | Secondary text, labels |
| `accent` | `#c8aa6e` | Brand accent, highlights |
| `success` | `#3fb950` | Connected, positive |
| `danger` | `#f85149` | Disconnected, errors |

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

Font family: system default for now. *Decide and document the font (and its license) here.*

## Naming rules
- kebab-case, same name everywhere (Pencil variable, this table, `Theme.<name>` in Slint).
- Name by role (`text-secondary`), not by look (`grey-400`).
