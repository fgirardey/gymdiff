---
status: proposed
date: 2026-11-08
---

# ADR-0009 Open-source license

## Context
The repository goes public to simplify the release and auto-update setup ([[ADR-0008 Versioning and releases]]): release assets download without a token, Actions minutes are free, `main` can be protected, and free code signing for open source (SignPath, see [[Code signing]]) becomes possible. Until now `LICENSE` held the Unlicense (public domain) while the README said "all rights reserved", so the actual terms were unclear. Some files in the repository are not ours to license: Riot's role symbols and champion portraits ([[ADR-0005 Bundle the role symbols]]) and one Lucide icon.

## Decision
**cooldown.lol's code and assets are licensed under MIT OR Apache-2.0, at the user's option**, with explicit exclusions listed in `THIRD-PARTY-NOTICES.md`. Decided with the author on 2026-11-08.

- Files: `LICENSE-MIT`, `LICENSE-APACHE`; `license = "MIT OR Apache-2.0"` in `[workspace.package]`, inherited by every crate.
- Contributions are accepted under the same dual license (inbound = outbound, stated in the README).
- Excluded:
  - Riot Games assets (role symbols, champion portraits): used under Riot's "Legal Jibber Jabber" policy, no rights granted;
  - the cooldown.lol name and logo (`design/brand/`, `ui/images/crest.svg`): forks use their own name and logo ([[Brand]]);
  - the Lucide settings icon: ISC, notice kept in `THIRD-PARTY-NOTICES.md`.
- Slint stays under its Royalty-free Desktop License ([[ADR-0001 Native UI with Slint]]), which allows any license for our code.

## Alternatives considered
- **Unlicense** (the previous file) — cannot cover the Riot assets, and gives up even attribution.
- **MIT alone** — fine, but no explicit patent grant; the dual license is the Rust ecosystem convention.
- **GPL-3.0** — keeps forks open, but heavier for contributors and users, and not needed for a free, non-commercial app.
- **All rights reserved** — public source nobody may reuse; little benefit over a private repository.

## Consequences
- Anyone may reuse the code, including commercially, with attribution. cooldown.lol itself stays free and non-commercial ([[Vision#Non-goals]]); Riot's policies apply to the app, not to someone reusing our code elsewhere.
- A new third-party file (icon, font, image) must be listed in `THIRD-PARTY-NOTICES.md` with its license, or replaced by our own.
- The origin of the other UI icons (`chevron-left`, `history`, `info`) and of the exercise icons should be confirmed and listed if they are third-party.
