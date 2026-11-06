# League APIs

GymDiff uses two **local** APIs exposed by League of Legends on `127.0.0.1`, and possibly the **Riot Games API** (remote, needs an API key) later.

> The LCU API is **not officially supported** by Riot. Endpoints can change or disappear with any patch. Deserialize leniently and never crash on unexpected data. See [[Riot policies]].

## LCU API (League Client Update)

Exposed by the League client (launcher + lobby UI) while it is running.

### Discovery: the lockfile
When the client starts it writes a file named `lockfile` in its installation folder:

```
LeagueClient:<pid>:<port>:<password>:<protocol>
```

Example: `LeagueClient:12345:54321:AbCdEf123:https`. The port and password change at every client start.

Typical locations (the install folder can be changed by the user):
| OS | Path |
|---|---|
| Windows | `C:\Riot Games\League of Legends\lockfile` |
| macOS | `/Applications/League of Legends.app/Contents/LoL/lockfile` |

Fallback (not implemented yet): read the `LeagueClientUx` process command line and extract `--app-port=` and `--remoting-auth-token=`.

Implementation: `gymdiff-lol/src/lockfile.rs`.

### REST
- Base URL: `https://127.0.0.1:<port>`
- Auth: HTTP Basic, user `riot`, password from the lockfile
- TLS: certificate signed by Riot's root CA (`riotgames.pem`, published on the Riot developer portal)

Endpoints used:
| Endpoint | Returns | Used for |
|---|---|---|
| `GET /lol-gameflow/v1/gameflow-phase` | JSON string: `None`, `Lobby`, `Matchmaking`, `ReadyCheck`, `ChampSelect`, `GameStart`, `InProgress`, `Reconnect`, `WaitingForStats`, `PreEndOfGame`, `EndOfGame` | [[Client status]] |

### WebSocket events (not used yet)
- URL: `wss://127.0.0.1:<port>/`, same Basic auth
- Protocol: WAMP 1.0 messages as JSON arrays
  - subscribe to everything: `[5, "OnJsonApiEvent"]`
  - subscribe to one endpoint: `[5, "OnJsonApiEvent_lol-gameflow_v1_session"]` (path without leading slash, `/` → `_`)
  - events arrive as `[8, "OnJsonApiEvent", { "uri": "...", "eventType": "Create|Update|Delete", "data": ... }]`

Moving from polling to this event stream is on the [[Roadmap]]; the simulator will need to emulate it too.

## Live Client Data API

Exposed by the **game process** only while a match is running (including spectator mode).

- Base URL: `https://127.0.0.1:2999`
- No auth. Same Riot root CA.
- Not listening outside a game (connection refused); answers 404 while the game is loading.

Endpoints (under `/liveclientdata/`):
| Endpoint | Content |
|---|---|
| `allgamedata` | everything below in one call |
| `activeplayer`, `activeplayername`, `activeplayerabilities`, `activeplayerrunes` | the local player |
| `playerlist` | all players (champion, team, items, scores, …) |
| `playerscores?summonerName=…`, `playersummonerspells?…`, `playermainrunes?…`, `playeritems?…` | one player |
| `eventdata` | game events (kills, dragons, …) |
| `gamestats` | `gameMode`, `gameTime`, `mapName`, `mapNumber`, `mapTerrain` |

Used today: `gamestats` → `gameTime` ([[Client status]]).

Implementation: `gymdiff-lol/src/live_client.rs` (models), `gymdiff-lol/src/client.rs` (HTTP).

## Riot Games API (remote)
Not used yet. Needs an API key from the Riot developer portal and a registered product. If an endpoint exists both in the Riot API and in the LCU, Riot requires using the Riot API.

## References
- Hextech Docs (community): <https://hextechdocs.dev/getting-started-with-the-lcu-api/>
- Riot developer portal: <https://developer.riotgames.com>
