# Riot policies

What GymDiff may and may not do. **This is a working summary, not legal advice.** Riot's policies change: re-read the sources below before designing any feature that runs during champion select or a game, and before any public release.

Last reviewed: 2026-11-07.

## Hard rules for this codebase
- Use only the **documented/tolerated interfaces**: LCU API, Live Client Data API, Riot Games API.
- **Never** read or write game memory, inject code, hook the game process, intercept network traffic, or automate inputs. Besides breaking the terms of service, Riot's anti-cheat (Vanguard) is designed to detect this kind of behaviour.
- Do not use the LCU to bypass Riot API rate limits. If an endpoint exists in both, use the Riot API.

## The recorder
`gymdiff-recorder` is a development tool, not a product feature: it only issues `GET` requests to the local APIs every 2 s or more (an allowlist of endpoints, no write calls), like the app. Recordings are anonymised before they reach the disk, never shown in the UI, and never sent anywhere. Do not record champion select in modes that hide names unless you keep the anonymisation: the LCU can expose names the normal client hides ([[Riot policies#Game integrity (general policy)|de-anonymisation rule]]).

## Vanguard and third-party tools
- Riot announced that game **memory access** is blocked by default for unknown third-party apps from 2026-10-06 (approved tools get a grace period until April 2027). GymDiff never reads memory, so this does not concern it. Source: Riot support, "game memory access removed for third party apps".
- Riot states that tools built on the LCU and in-game APIs keep working, but its Developer Relations team does not own those APIs and cannot guarantee them.
- Nothing found says a player is banned for running a read-only local-API tool; bans target scripting, botting and memory/injection tools. This is an observation, not a guarantee.

## Registration
- Any product serving players must be **registered on the Riot developer portal**, even if it only uses the local APIs.
- Keep the registration up to date with the product.

## Game integrity (general policy)
Products must not:
- alter the goal of the game or **create an unfair advantage**,
- **de-anonymize** players who cannot be identified from visible information (e.g. hidden names in ranked champion select / anonymous mode),
- expose a player's historic Riot IDs,
- create an alternative to official skill rankings (custom MMR/ELO displays),
- **dictate player decisions** (highlighting a decision and offering options is fine),
- scout opponents' champions, bypass a skill test, or provide **dynamic real-time information** that gives an edge.

## In-game overlays
- Riot tightened its overlay rules in May 2025 (as reported publicly): no ultimate cooldown trackers, no overlays mimicking Riot's UI, ads must be pre-approved, no custom rank/MMR displays, no private data in anonymous mode. Summoner spell and jungle timers were reported as "under review" — treat them as **not allowed** until confirmed.
- The developer policy only explicitly approves overlays showing **static data available before the game**.
- Rule of thumb: if a feature tells the player something about opponents they could not see themselves, or tracks timers for them, it is out.

## Monetization
GymDiff will never be monetized ([[Vision#Non-goals]]). For reference:
- A **free tier** is mandatory; paid content must be transformative.
- No betting or gambling.
- The product must be approved/acknowledged on the developer portal before monetizing.

## Branding
- Include Riot's "Legal Jibber Jabber" disclaimer (done in `README.md`; add it to the About screen and any website).
- Do not use Riot logos or present GymDiff as official.

## Checklist for a new feature
- [ ] Does it run during champion select or a game? Which data does it show?
- [ ] Could any of that data be unavailable to the player in the normal client? → not allowed
- [ ] Does it track timers, cooldowns or opponents? → not allowed
- [ ] Does it tell the player what to do instead of offering options? → rework
- [ ] Does it use only the LCU / Live Client Data / Riot API?

## Sources
- Riot Developer API Policy: <https://support-developer.riotgames.com/hc/en-us/articles/22698732381587>
- Riot developer portal: <https://developer.riotgames.com>
- Overwolf's League of Legends compliance guide (useful summary): <https://dev.overwolf.com/ow-native/guides/game-compliance/league-of-legends>
- Report on the May 2025 overlay changes: <https://lolnow.gg/riot-cracks-down-on-overlays-in-league-of-legends-a-long-overdue-clean-up/>
