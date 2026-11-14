---
paths:
  - "crates/cooldown-lol/**"
  - "fixtures/**"
---

# cooldown-lol and API payloads

- The LCU API is unsupported by Riot and can change without notice. Deserialize leniently: ignore unknown fields, keep unknown enum values (`Unknown(String)`, see `GamePhase`), model only the fields cooldown.lol uses.
- Every new API payload gets a fixture-based parsing test.
- The app reaches the game only through the `LolClient` trait (`client.rs`); `HttpLolClient` (`http_client.rs`) serves both the real client and the simulator.
- Endpoint reference: `docs/Architecture/League APIs.md`. Update it when you use a new endpoint.
