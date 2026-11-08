# Roadmap

## Now — foundations
- [x] Workspace skeleton (core / lol / simulator / app)
- [x] Lockfile discovery, gameflow phase, in-game clock ([[Client status]])
- [x] Simulator with scenario replay
- [x] CI on Windows and macOS with a Windows build artifact
- [x] Write the [[Vision]] (draft)
- [x] Create `design/gymdiff.pen` with the `Foundations` page and real [[Tokens]]
- [x] Brand: logo and app icon ([[Brand]])
- [x] App shell with the M0 screens: [[First launch]] welcome, empty [[History]], About; macOS and Windows release builds in CI
- [ ] Bundle the brand fonts (Barlow Condensed, Inter) in the app
- [ ] Windows app icon (`.ico` from `design/brand/`) embedded in the executable
- [ ] First validation on the Windows PC (checklist in [[Windows testing]])
- [ ] Register the project on the Riot developer portal ([[Riot policies]])

## Next — technical
- [ ] Trust Riot's root certificate (`riotgames.pem`) instead of accepting any certificate
- [ ] LCU WebSocket event stream (WAMP) instead of polling, emulated by the simulator
- [ ] Process-based client discovery (fallback when the install folder is not the default)
- [x] `gymdiff-recorder`: record real sessions on Windows into scenarios ([[Recorder]]); WebSocket events not recorded yet
- [ ] Overlay window on Windows (always on top, click-through, positioned over the game)
- [x] Settings persistence (`settings.json`, [[ADR-0007 Local persistence]])
- [ ] Windows installer and code signing (options and status: [[Code signing]])
- [ ] Auto-update

## MVP — post-game workout
- [ ] Validate [[Vision]], [[Workout formula]] and [[Post-game workout]] (open questions)
- [ ] Record a real end-of-game payload and champ select session on Windows
- [x] Sketch the frames listed in [[Screens]] in Pencil
- [ ] Validate the sketches, then mark the features `designed`
- [x] [[First launch]] welcome screen (first persisted flag)
- [ ] Core: workout formula, sessions, with tests
- [ ] LCU: end-of-game stats, champ select role, simulator scenarios
- [ ] UI: workout screen and its states
- [ ] [[History]]: persisted games, sessions split on 1-hour breaks, timeline screen

## Later — product
- [ ] Settings: difficulty multiplier, custom targets, exercise choice
- [ ] ARAM and other modes (a recipe without roles)
- [ ] In-game overlay — only after checking [[Riot policies]]: the developer policy only explicitly approves static pre-game data
