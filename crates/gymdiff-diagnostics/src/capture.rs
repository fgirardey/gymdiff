//! Keeps the latest warnings and errors in memory, scrubbed, for a report.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

use crate::report::Entry;
use crate::scrub::scrub;

/// Longest message kept, in characters.
const MAX_MESSAGE_CHARS: usize = 500;

/// Bounded, shared buffer of scrubbed `WARN`/`ERROR` entries (oldest dropped).
#[derive(Debug, Clone)]
pub struct ErrorBuffer {
    capacity: usize,
    entries: Arc<Mutex<VecDeque<Entry>>>,
}

impl ErrorBuffer {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Arc::default(),
        }
    }

    /// The `tracing` layer feeding this buffer.
    #[must_use]
    pub fn layer(&self) -> CaptureLayer {
        CaptureLayer {
            buffer: self.clone(),
        }
    }

    /// Copy of the current entries, oldest first.
    #[must_use]
    pub fn snapshot(&self) -> Vec<Entry> {
        self.lock().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.lock().clear();
    }

    fn push(&self, entry: Entry) {
        let mut entries = self.lock();
        if entries.len() >= self.capacity {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    /// Logging must never crash the app: recover from a poisoned lock.
    fn lock(&self) -> std::sync::MutexGuard<'_, VecDeque<Entry>> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// `tracing` layer that records `WARN` and `ERROR` events into an [`ErrorBuffer`].
#[derive(Debug, Clone)]
pub struct CaptureLayer {
    buffer: ErrorBuffer,
}

impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        if *metadata.level() > Level::WARN {
            return;
        }
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message: String = scrub(&visitor.text)
            .chars()
            .take(MAX_MESSAGE_CHARS)
            .collect();
        self.buffer.push(Entry {
            level: metadata.level().to_string(),
            target: metadata.target().to_owned(),
            message,
        });
    }
}

/// Renders the message first, then `name=value` fields.
#[derive(Default)]
struct MessageVisitor {
    text: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.text, "{value:?}");
        } else {
            let _ = write!(self.text, " {}={value:?}", field.name());
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::layer::SubscriberExt;

    use super::*;

    fn with_buffer(capacity: usize, f: impl FnOnce()) -> ErrorBuffer {
        let buffer = ErrorBuffer::new(capacity);
        let subscriber = tracing_subscriber::registry().with(buffer.layer());
        tracing::subscriber::with_default(subscriber, f);
        buffer
    }

    #[test]
    fn keeps_only_warnings_and_errors() {
        let buffer = with_buffer(10, || {
            tracing::info!("fine");
            tracing::debug!("fine too");
            tracing::warn!("careful");
            tracing::error!("broken");
        });
        let levels: Vec<_> = buffer.snapshot().into_iter().map(|e| e.level).collect();
        assert_eq!(levels, ["WARN", "ERROR"]);
    }

    #[test]
    fn scrubs_messages_and_fields_at_capture() {
        let buffer = with_buffer(10, || {
            tracing::error!(path = r"C:\Users\Florian\x", "cannot open for Faker#KR1");
        });
        let message = &buffer.snapshot()[0].message;
        assert!(
            !message.contains("Florian") && !message.contains("Faker"),
            "{message}"
        );
    }

    #[test]
    fn drops_the_oldest_when_full() {
        let buffer = with_buffer(2, || {
            tracing::warn!("one");
            tracing::warn!("two");
            tracing::warn!("three");
        });
        let messages: Vec<_> = buffer.snapshot().into_iter().map(|e| e.message).collect();
        assert_eq!(messages, ["two", "three"]);
    }
}
