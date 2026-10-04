/// Custom tracing layer that captures log entries into the in-app log panel.
///
/// Works alongside the standard env-filter subscriber.
use std::sync::{Arc, Mutex};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

/// Shared log buffer type.
pub type LogBuffer = Arc<Mutex<Vec<String>>>;

/// A tracing [`Layer`] that appends formatted records to a shared buffer.
pub struct InAppLogLayer {
    buffer: LogBuffer,
}

impl InAppLogLayer {
    pub fn new(buffer: LogBuffer) -> Self {
        Self { buffer }
    }
}

impl<S: Subscriber> Layer<S> for InAppLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        use tracing_subscriber::field::Visit;

        struct Visitor(String);
        impl Visit for Visitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{:?}", value).trim_matches('"').to_string();
                }
            }
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" {
                    self.0 = value.to_string();
                }
            }
        }

        let mut visitor = Visitor(String::new());
        event.record(&mut visitor);

        let level = *event.metadata().level();
        let target = event.metadata().target();
        let entry = format!("[{level}] {target}: {}", visitor.0);

        if let Ok(mut buf) = self.buffer.lock() {
            buf.push(entry);
            // Cap to 1000.
            if buf.len() > 1000 {
                buf.drain(..100);
            }
        }
    }
}
