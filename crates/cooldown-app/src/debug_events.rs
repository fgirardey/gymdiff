//! THROWAWAY: events fired from the debug screen (`ui/screens/debug.slint`),
//! or at start with `COOLDOWN_TEST_CRASH=<kind>`, to test the error reports end
//! to end (the variable works in release builds too, to check that a CI build
//! is symbolicated). Remove with the screen.

/// Fires the event named `kind` (see `DebugScreen`'s `fire` callback).
pub fn fire(kind: &str) {
    match kind {
        "warning" => tracing::warn!("debug: test warning from the debug screen"),
        "error" => tracing::error!("debug: test error from the debug screen"),
        "scrubbed-error" => tracing::error!(
            "debug: cannot open C:\\Users\\Florian\\cooldown.log for Florian#EUW from 192.168.1.42 (florian@example.com)"
        ),
        "panic" => {
            // In a thread: a panic on the UI thread would take the window down.
            let spawned = std::thread::Builder::new()
                .name("debug-panic".into())
                .spawn(|| panic!("debug: test panic from the debug screen"));
            if let Err(error) = spawned {
                tracing::warn!(%error, "cannot start the debug panic thread");
            }
        }
        // A native crash without `unsafe`: Rust's `abort` is a fast-fail on
        // Windows that no crash handler sees, a stack overflow is not.
        "stack-overflow" => {
            tracing::warn!("debug: overflowing the stack");
            std::hint::black_box(overflow(0));
        }
        other => tracing::warn!(kind = other, "unknown debug event"),
    }
}

/// Fires the event named by `COOLDOWN_TEST_CRASH`, if set.
pub fn fire_from_env() {
    if let Ok(kind) = std::env::var("COOLDOWN_TEST_CRASH") {
        fire(&kind);
    }
}

/// Recurses until the stack is exhausted; `black_box` keeps the frames.
fn overflow(depth: u64) -> u64 {
    let frame = std::hint::black_box([depth; 128]);
    if std::hint::black_box(depth) == u64::MAX {
        return 0;
    }
    overflow(frame[0] + 1) + frame[1]
}
