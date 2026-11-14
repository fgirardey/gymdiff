## What and why
<!-- What the user can now do or see, and why. Link the feature note: docs/Product/Features/<Feature>.md -->

## Screenshots
<!-- UI changes only: paste the table printed by `just pr-screenshots` (one row per Pencil frame).
     Add a recording or a `just dev` capture for flows that a still image doesn't show.
     No UI change: write "No UI change." -->

## How to try it
<!-- e.g. `just dev`, `COOLDOWN_DATA_DIR=$(mktemp -d) just dev` (first launch), `just preview screens/history` -->

## Notes for the reviewer
<!-- Where to start reading, trade-offs, what is left for a follow-up. Delete if empty. -->

## Checklist
- [ ] `just check` passes
- [ ] Docs updated in the same change (`docs/`), new ADR for a significant decision
- [ ] Every changed screen or component still matches its Pencil frame (`just preview <path> <Story>`)
- [ ] Nothing new is shown during a game, or `docs/Compliance/Riot policies.md` was checked
- [ ] Platform code or real-client behaviour: tested on Windows
