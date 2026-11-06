---
status: accepted
date: 2026-11-06
---

# ADR-0002 Docs and design as sources of truth

## Context
GymDiff is implemented mostly with Claude Code. Conversations are ephemeral: decisions, product intent and visual design must live in durable artifacts that both a human and Claude can read, version and diff, independently of any AI tool.

## Decision
Keep three kinds of artifacts in the repository, next to the code:

1. **Product and decisions in Markdown**, in `docs/`, edited with **Obsidian** (the folder is a vault).
2. **Visual design in Pencil**, in `design/*.pen` (JSON files). Claude reads them through **Pencil's MCP server**.
3. **Design tokens in both places**: [[Tokens]] (human-readable reference) and `crates/gymdiff-app/ui/theme.slint` (used by the code). They must stay identical.

Claude Code implements **only** from these artifacts (rules in `CLAUDE.md`). When they are missing or contradictory, it asks instead of guessing.

## Alternatives considered
- **Figma** — the most capable MCP for reading designs, but designs live in Figma's cloud behind a subscription, outside Git.
- **Penpot** — free and open source, but its MCP works through a plugin that must stay open in a browser tab on the current page, which is impractical as a memory Claude consults while coding.
- **Designs only in conversation / generated directly as code** — no durable memory, no review of the design before implementation.

## Consequences
- Every feature starts as a note ([[Spec to code]]), then frames in Pencil, then code.
- Design changes are reviewed in Git like code. `.pen` files are never merged by hand (see `design/README.md`).
- The Pencil app must be running with the file open for Claude to use the MCP. Claude can still read the `.pen` JSON directly as a fallback.
- Pencil is a young tool (free at the time of writing, may become paid). Because files are local and in Git, switching tools later means re-creating designs, but no data is lost.
