//! One cooldown.lol per data folder: a second cooldown.lol asks the running one to show
//! its window, then exits (`docs/Product/Features/Running in the background.md`,
//! ADR-0013).
//!
//! The running cooldown.lol listens on a local socket named after the data folder:
//! a named pipe on Windows, a socket file in the data folder elsewhere. Each
//! Windows account (and each `COOLDOWN_DATA_DIR`) has its own.

use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, ListenerOptions, Name, Stream,
};

/// What [`claim`] found.
pub enum SingleInstance {
    /// No other cooldown.lol: this one listens, on its own thread, until exit.
    First,
    /// Another cooldown.lol runs and was told what to do: exit now.
    AlreadyRunning,
}

/// Becomes the running cooldown.lol, or hands over to the one already running.
///
/// When another cooldown.lol answers, it is asked to show its window unless
/// `background` (a start with Windows has nothing to show). Otherwise this one
/// listens and calls `show` for every later cooldown.lol that asks.
/// Without a working socket cooldown.lol still starts: a second instance is better
/// than none.
pub fn claim(
    data_dir: &Path,
    background: bool,
    show: impl Fn() + Send + 'static,
) -> SingleInstance {
    let name = match socket_name(data_dir) {
        Ok(name) => name,
        Err(error) => {
            tracing::warn!(%error, "no single-instance socket name");
            return SingleInstance::First;
        }
    };
    if let Ok(mut running) = Stream::connect(name.borrow()) {
        let message = if background { BACKGROUND } else { SHOW };
        if let Err(error) = running.write_all(message) {
            tracing::warn!(%error, "cannot reach the running cooldown.lol");
        }
        tracing::info!(background, "cooldown.lol already runs: handing over");
        return SingleInstance::AlreadyRunning;
    }
    if let Err(error) = listen(name, show) {
        tracing::warn!(%error, "cannot listen for other cooldown.lol instances: several may run");
    }
    SingleInstance::First
}

/// Asks the running cooldown.lol to show its window.
const SHOW: &[u8] = b"show\n";
/// Tells the running cooldown.lol that a background start found it: nothing to do.
const BACKGROUND: &[u8] = b"background\n";

fn listen(name: Name<'static>, show: impl Fn() + Send + 'static) -> io::Result<()> {
    let listener = ListenerOptions::new()
        .name(name)
        // A socket file left by a cooldown.lol that crashed: nobody answered it above.
        .try_overwrite(true)
        .create_sync()?;
    std::thread::Builder::new()
        .name("single-instance".into())
        .spawn(move || {
            for connection in listener.incoming().filter_map(Result::ok) {
                let mut line = String::new();
                if BufReader::new(connection).read_line(&mut line).is_ok()
                    && line.as_bytes() == SHOW
                {
                    tracing::info!("another cooldown.lol asked to show the window");
                    show();
                }
            }
        })?;
    Ok(())
}

/// The socket of the cooldown.lol using `data_dir`.
fn socket_name(data_dir: &Path) -> io::Result<Name<'static>> {
    if GenericNamespaced::is_supported() {
        let id = fnv1a(data_dir.as_os_str().as_encoded_bytes());
        format!("cooldown-{id:016x}").to_ns_name::<GenericNamespaced>()
    } else {
        std::fs::create_dir_all(data_dir)?;
        data_dir.join(SOCKET_FILE).to_fs_name::<GenericFilePath>()
    }
}

/// Socket file in the data folder, where named sockets do not exist (macOS).
const SOCKET_FILE: &str = "instance.sock";

/// A hash that stays the same across Rust versions, so two cooldown.lol builds
/// agree on the name.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn fnv1a_matches_the_reference() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn a_second_instance_asks_the_first_to_show() {
        let dir = std::env::temp_dir().join(format!("cooldown-instance-{}", std::process::id()));
        let (shown, showings) = mpsc::channel();

        let first = claim(&dir, false, move || shown.send(()).unwrap());
        assert!(matches!(first, SingleInstance::First));

        let background = claim(&dir, true, || panic!("not the running one"));
        assert!(matches!(background, SingleInstance::AlreadyRunning));
        let second = claim(&dir, false, || panic!("not the running one"));
        assert!(matches!(second, SingleInstance::AlreadyRunning));

        // Only the normal start asks to show the window.
        showings.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(showings.recv_timeout(Duration::from_millis(200)).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
