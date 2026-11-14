---
paths:
  - "crates/cooldown-core/**"
---

# cooldown-core

- Pure domain logic: no I/O (network, filesystem, processes), no async, no `#[cfg(target_os)]`. If it can't be unit-tested on any machine, it doesn't belong here.
- State changes go through the reducer `CompanionState::apply(Event) -> CompanionState`. New input from the outside world = a new `Event` variant (`event.rs`), never a direct mutation from a data source.
- Every behaviour gets a unit test in the same file.
