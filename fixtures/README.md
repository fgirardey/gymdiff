# Fixtures

Data used by the simulator and by tests.

```
fixtures/
└── scenarios/
    └── <scenario-name>/
        └── scenario.json
```

- `synthetic-*` scenarios are hand-written. They follow the shape of the real APIs but are **not** recordings.
- Recorded scenarios are captured on the Windows PC with `just record <date>-<what>` (see `docs/Workflow/Recorder.md`) and live in `recorded-<date>-<what>/`.
- Every `scenario.json` is validated by `cargo test` (`cooldown-simulator` → `bundled_scenarios_are_valid`).

Format: see [`docs/Workflow/Simulator.md`](../docs/Workflow/Simulator.md).

> The recorder anonymises names and identifiers before writing, but **read a recording before committing it**: it is match data from real games.
