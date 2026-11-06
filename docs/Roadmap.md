# Roadmap

## Now — foundations
- [x] Workspace skeleton (core / lol / simulator / app)
- [x] Lockfile discovery, gameflow phase, in-game clock ([[Client status]])
- [x] Simulator with scenario replay
- [x] CI on Windows and macOS with a Windows build artifact
- [x] Write the [[Vision]] (draft)
- [x] Create `design/gymdiff.pen` with the `Foundations` page and real [[Tokens]]
- [x] Brand: logo and app icon ([[Brand]])
- [ ] Bundle the brand fonts (Barlow Condensed, Inter) in the app
- [ ] Windows app icon (`.ico` from `design/brand/`) embedded in the executable
- [ ] First validation on the Windows PC (checklist in [[Windows testing]])
- [ ] Register the project on the Riot developer portal ([[Riot policies]])

## Next — technical
- [ ] Trust Riot's root certificate (`riotgames.pem`) instead of accepting any certificate
- [ ] LCU WebSocket event stream (WAMP) instead of polling, emulated by the simulator
- [ ] Process-based client discovery (fallback when the install folder is not the default)
- [ ] `gymdiff-recorder`: record real sessions on Windows into scenarios
- [ ] Overlay window on Windows (always on top, click-through, positioned over the game)
- [ ] Settings persistence
- [ ] Windows installer and code signing
- [ ] Auto-update

## MVP — post-game workout
- [ ] Validate [[Vision]], [[Workout formula]] and [[Post-game workout]] (open questions)
- [ ] Record a real end-of-game payload and champ select session on Windows
- [ ] Design the `Post-game workout` frames in Pencil
- [ ] Core: workout formula + session total, with tests
- [ ] LCU: end-of-game stats, champ select role, simulator scenarios
- [ ] UI: workout screen and its states

## Later — product
- [ ] Settings: difficulty multiplier, custom targets, exercise choice
- [ ] Persist history across restarts (daily / weekly totals)
- [ ] ARAM and other modes (a recipe without roles)
- [ ] In-game overlay — only after checking [[Riot policies]]: the developer policy only explicitly approves static pre-game data
