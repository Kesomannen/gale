use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Listener};
use tracing::{debug, error, trace};

#[derive(Debug)]
pub struct EventBuffer(Arc<Inner>);

#[derive(Debug)]
struct Inner {
    frontend_ready: AtomicBool,
    buffer: Mutex<Vec<BufferedEvent>>,
    app: AppHandle,
}

#[derive(Debug)]
struct BufferedEvent {
    event: String,
    payload: String,
}

impl EventBuffer {
    pub fn new(app: AppHandle) -> Self {
        let inner = Arc::new(Inner {
            frontend_ready: AtomicBool::new(false),
            buffer: Mutex::new(Vec::new()),
            app,
        });

        let inner_clone = Arc::clone(&inner);
        inner.app.listen("ready", move |_evt| {
            inner_clone.on_frontend_ready();
        });

        Self(inner)
    }

    pub fn emit(&self, event: impl Into<String>, payload: &impl Serialize) {
        let content = match serde_json::to_string(payload) {
            Ok(json) => json,
            Err(err) => {
                error!(
                    ?err,
                    event = event.into(),
                    "failed to serialize event payload",
                );
                return;
            }
        };

        self.0.enqueue(event.into(), content.into());
    }
}

impl Inner {
    fn enqueue(&self, event: String, payload: String) {
        if self.frontend_ready.load(Ordering::SeqCst) {
            self.emit_now(&event, payload);
        } else {
            let mut buffer = self.buffer.lock().unwrap();
            trace!(
                event,
                buffer_len = buffer.len() + 1,
                "buffering event until frontend is ready"
            );
            buffer.push(BufferedEvent { event, payload });
        }
    }

    fn on_frontend_ready(&self) {
        self.frontend_ready.store(true, Ordering::SeqCst);

        let mut buffer = self.buffer.lock().unwrap();

        debug!(
            buffer_len = buffer.len(),
            "frontend is ready, emitting buffered events"
        );

        for event in buffer.drain(..) {
            self.emit_now(&event.event, event.payload);
        }
    }

    fn emit_now(&self, event: &str, payload: String) {
        if let Err(err) = self.app.emit_str(event, payload) {
            error!(?err, "failed to emit event to frontend");
        } else {
            trace!(event, "emitted event to frontend");
        }
    }
}
