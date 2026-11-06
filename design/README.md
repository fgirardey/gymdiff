# Design

Pencil design files for GymDiff. This folder is the **design memory** of the project: Claude Code reads it through the Pencil MCP server when implementing UI.

## Files

| File | Content |
|---|---|
| `gymdiff.pen` | Main design file |
| `brand/` | Logo files exported from the `Logo / *` components (see [`docs/Design/Brand.md`](../docs/Design/Brand.md)) |

Add more `.pen` files only if the main one becomes hard to navigate (e.g. `overlay.pen` for in-game overlays).

## Conventions

So that Claude (and you, in six months) can find things:

- **One page per area**: `Foundations`, `Components`, then one page per feature.
- **Frame names mirror the feature notes** in `docs/Product/Features/`: `<Feature> / <Screen> / <State>`, e.g. `Champ Select / Main / Loading`. Every state the UI can be in gets a frame (empty, loading, error, nominal).
- **Real sizes**: frames use the actual window or overlay size in logical pixels. Note the target resolution in the frame name when it matters (`… @1080p`).
- **Tokens first**: colors, spacing, radius and type sizes come from the `Foundations` page and are mirrored in [`docs/Design/Tokens.md`](../docs/Design/Tokens.md). Do not use one-off values.
- **Annotations**: write behaviour that a static frame cannot show (animations, hover, what updates live) as text notes next to the frame, or in the feature note.
- **No Riot assets** (logos, splash art, icons) in committed designs unless their use is allowed — see [`docs/Compliance/Riot policies.md`](../docs/Compliance/Riot%20policies.md). Use neutral placeholders.

## Git

`.pen` files are JSON. Commit them like code, with a message describing the design change. Do not merge conflicting `.pen` files by hand: keep one version and reapply the other change in Pencil (`.gitattributes` marks them as non-mergeable).

See [`docs/Workflow/Design with Pencil.md`](../docs/Workflow/Design%20with%20Pencil.md).
