# Roadmap

## Now — foundations
- [x] Workspace skeleton (core / lol / app)
- [x] Lockfile discovery, gameflow phase, in-game clock ([[Client status]])
- [x] CI on Windows and macOS with a Windows build artifact
- [x] Write the [[Vision]] (draft)
- [x] Create `design/cooldown.pen` with the `Foundations` page and real [[Tokens]]
- [x] Brand: logo and app icon ([[Brand]])
- [x] App shell with the M0 screens: [[First launch]] welcome, empty [[History]], About; macOS and Windows release builds in CI
- [ ] Bundle the brand fonts (Barlow Condensed, Inter) in the app
- [x] Windows app icon (`.ico` from `design/brand/`) embedded in the executable, crest as the window icon, no console window in release builds
- [ ] First validation on the Windows PC (checklist in [[Windows testing]])
- [ ] Register the project on the Riot developer portal ([[Riot policies]])

## Next — technical
- [ ] Trust Riot's root certificate (`riotgames.pem`) instead of accepting any certificate
- [ ] LCU WebSocket event stream (WAMP) instead of polling
- [ ] Process-based client discovery (fallback when the install folder is not the default)
- [ ] Overlay window on Windows (always on top, click-through, positioned over the game)
- [x] Settings persistence (`settings.json`, [[ADR-0007 Local persistence]])
- [x] Versioning (SemVer + build number), releases with release-please, symbolicated Sentry issues ([[ADR-0008 Versioning and releases]])
- [x] Windows installer with Velopack ([[Installation]], [[ADR-0011 Windows installer and updates with Velopack]])
- [x] Auto-update of alpha builds from `main` ([[Auto-update]])
- [ ] Code signing of the installer and updates (SignPath Foundation, now that the repository is public: [[Code signing]])
- [ ] Stable update channel for friends (tagged releases, once signed)

## MVP — post-game workout
- [ ] Validate [[Vision]], [[Workout formula]] and [[Post-game workout]] (open questions)
- [ ] Record a real end-of-game payload and champ select session on Windows
- [x] Sketch the frames listed in [[Screens]] in Pencil
- [ ] Validate the sketches, then mark the features `designed`
- [x] [[First launch]] welcome screen (first persisted flag)
- [ ] Core: workout formula, sessions, with tests
- [ ] LCU: end-of-game stats, champ select role
- [ ] UI: workout screen and its states
- [ ] [[History]]: persisted games, sessions split on 1-hour breaks, timeline screen

## Later — product
- [ ] Settings: difficulty multiplier, custom targets, exercise choice
- [ ] ARAM and other modes (a recipe without roles)
- [ ] In-game overlay — only after checking [[Riot policies]]: the developer policy only explicitly approves static pre-game data
