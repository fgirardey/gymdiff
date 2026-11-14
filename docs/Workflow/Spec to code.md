# Spec to code

How a feature travels from idea to a validated Windows build.

| Step | Artifact | Who | Done when |
|---|---|---|---|
| 1. Idea | line in [[Roadmap]] | you | — |
| 2. Spec | `Product/Features/<Feature>.md` from `_templates/Feature.md` | you (+ Claude to challenge it) | behaviour covers every state, compliance checked ([[Riot policies]]) |
| 3. Design | frames `<Feature> / <Screen> / <State>` in `design/cooldown.pen` | you (+ Claude via Pencil MCP) | every state has a frame, tokens only |
| 4. Decision (if any) | `Decisions/ADR-XXXX ….md` | you + Claude | status `accepted` |
| 5. Implementation | code, tests | Claude Code | `just check` passes, works in `just dev` |
| 6. Windows validation | checklist in the feature note | you, on the PC | boxes ticked, status `validated on Windows` |

## Prompting Claude Code
Point it at the artifacts rather than re-explaining:

> Implement [[Client status]] as specified in `docs/Product/Features/Client status.md`, using the frames `Client status / Main / *` in `design/cooldown.pen`. Follow CLAUDE.md.

Ask it to list open questions before writing code when the feature is large.

## Keeping the memory alive
- Behaviour changed during implementation? Update the feature note in the same commit.
- A trade-off was made? Write an ADR.
- A new token appeared? [[Tokens]] + `theme.slint` + Pencil.
