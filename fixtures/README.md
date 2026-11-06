# Fixtures

Data used by the simulator and by tests.

```
fixtures/
└── scenarios/
    └── <scenario-name>/
        └── scenario.json
```

- `synthetic-*` scenarios are hand-written. They follow the shape of the real APIs but are **not** recordings.
- Recorded scenarios (captured on Windows during real sessions, see `docs/Roadmap.md`) will be named `recorded-<date>-<what>`.
- Every `scenario.json` is validated by `cargo test` (`gymdiff-simulator` → `bundled_scenarios_are_valid`).

Format: see [`docs/Workflow/Simulator.md`](../docs/Workflow/Simulator.md).

> Recordings contain your summoner name and match data. Strip or anonymise anything personal before committing them.
