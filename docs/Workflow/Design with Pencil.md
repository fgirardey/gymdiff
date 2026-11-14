# Design with Pencil

[Pencil](https://pencil.dev) is the design tool of cooldown.lol (see [[ADR-0002 Docs and design as sources of truth]]). Its files (`design/*.pen`) are the design memory of the project; Claude Code reads them through Pencil's MCP server.

## Setup
1. Install the Pencil desktop app.
2. Create `design/cooldown.pen` (*New file*, save it in the repository's `design/` folder).
3. Connect Claude Code to Pencil's MCP server following Pencil's docs: <https://docs.pencil.dev/ai-integration>.
4. Check: with the file open in Pencil, ask Claude Code to list the pages and frames of `design/cooldown.pen`.

## Organising the file
Conventions are in `design/README.md`. Summary:
- Sections, top to bottom: `Foundations`, `Primitives`, `Core components`, then one row of screens per flow.
- Frames: `<Feature> / <Screen> / <State>`, matching the notes in [[Features]].
- Every UI state gets a frame (empty, loading, nominal, error).
- Values come from [[Tokens]].

## Designing with Claude
You can design by hand, or ask Claude (in Claude Code, with the MCP connected) to draft frames in Pencil from a feature note. Either way, **you** validate the design before implementation starts, and the feature note's `design:` field points to the frames.

## From design to Slint
Pencil does not export Slint. Claude Code:
1. reads the frame through the MCP (hierarchy, sizes, spacing, colors, text),
2. maps values to `Theme.*` tokens (adds missing tokens to [[Tokens]] and `theme.slint` first),
3. writes or updates the `.slint` component,
4. you compare with `just preview <path> <Story>` (live reload, see [[Previews]]) side by side with Pencil.

## Versioning
Commit `.pen` files with a descriptive message (`docs(design): add the champ select loading state`). Never merge two versions of a `.pen` file by hand.
