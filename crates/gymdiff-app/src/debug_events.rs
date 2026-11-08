//! THROWAWAY: events fired from the debug screen (`ui/screens/debug.slint`) to
//! test the error reports end to end. Remove with the screen.

/// Fires the event named `kind` (see `DebugScreen`'s `fire` callback).
pub fn fire(kind: &str) {
    match kind {
        "warning" => tracing::warn!("debug: test warning from the debug screen"),
        "error" => tracing::error!("debug: test error from the debug screen"),
        "scrubbed-error" => tracing::error!(
            "debug: cannot open C:\\Users\\Florian\\gymdiff.log for Florian#EUW from 192.168.1.42 (florian@example.com)"
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
        other => tracing::warn!(kind = other, "unknown debug event"),
    }
}
