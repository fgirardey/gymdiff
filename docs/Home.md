# GymDiff — Home

Open the `docs/` folder as an **Obsidian vault**. This vault is the written memory of the project: Claude Code reads it to implement features (see `CLAUDE.md` at the repository root).

## Product
- [[Vision]] — what GymDiff is, for whom, and why
- [[Features]] — index of feature notes
- [[Screens]] — window, top bar, screens and their Pencil frames
- [[Champion portraits]] — played and linked champion
- [[Workout formula]] — how a game becomes burpees, squats and plank
- [[Roadmap]] — what comes next

## Design
- [[Brand]] — logo, wordmark, app icon and usage rules
- [[Tokens]] — colors, spacing, typography (mirrored in `ui/theme.slint`)
- [[Design with Pencil]] — how the Pencil design files are organised

## Architecture
- [[Overview]] — crates, data flow, threading
- [[League APIs]] — LCU, Live Client Data, lockfile, WebSocket

## Decisions
- [[ADR-0001 Native UI with Slint]]
- [[ADR-0002 Docs and design as sources of truth]]
- [[ADR-0003 macOS-first development with a simulator]]
- [[ADR-0004 Champion images from Data Dragon]]
- [[ADR-0005 Bundle the role symbols]]
- [[ADR-0006 Opt-in error reports]]
- [[ADR-0007 Local persistence]]
- [[ADR-0008 Versioning and releases]]

## Workflow
- [[Development]] — the daily loop on macOS
- [[Simulator]] — scenarios and how to write them
- [[Previews]] — stories, playgrounds and galleries of the Slint UI
- [[Windows testing]] — building and testing on the Windows PC
- [[Spec to code]] — how a feature goes from idea to implementation
- [[Code organisation]] — one logical unit per file, public API first
- [[Logging]] — picking a log level: state changes, not ticks
- [[Pull requests]] — the PR template and generated UI screenshots

## Compliance
- [[Riot policies]] — what we can and cannot do
- [[Privacy]] — what stays local, what error reports contain, how to stop them

## Conventions for this vault
- English only.
- One note per feature, per decision, per workflow. Link generously with `[[wikilinks]]`.
- Templates live in `_templates/` (Obsidian: *Settings → Core plugins → Templates → Template folder location* = `_templates`).
- Decisions are never edited after being accepted: write a new ADR that supersedes the old one.
