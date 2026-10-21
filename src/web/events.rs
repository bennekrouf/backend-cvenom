//! Live change notifications, pushed to the browser over Server-Sent Events.
//!
//! Mutating routes publish a `CvEvent` once their change is on disk; `GET /events`
//! streams the events of the authenticated user only. Changes made outside the
//! browser (MCP tools via the api0 gateway, another tab) reach the open studio
//! without polling.
//!
//! The bus is an in-process broadcast channel: the server runs as one instance.

use crate::auth::AuthenticatedUser;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::Serialize;
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{self, error::RecvError};
use rocket::tokio::time::Duration;
use rocket::{get, Shutdown, State};

/// Buffer per subscriber. A client that falls further behind gets a `resync`.
const CHANNEL_CAPACITY: usize = 256;
const HEARTBEAT: Duration = Duration::from_secs(20);

#[derive(Clone, Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CvEvent {
    /// `files_changed`, `cv_generated` or `resync`.
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

impl CvEvent {
    /// Profiles or their files were created, changed or deleted.
    pub fn files_changed(profile: Option<String>) -> Self {
        Self { kind: "files_changed", profile, download_url: None, filename: None }
    }

    /// A PDF was generated and is ready to download.
    pub fn cv_generated(profile: String, download_url: String, filename: String) -> Self {
        Self {
            kind: "cv_generated",
            profile: Some(profile),
            download_url: Some(download_url),
            filename: Some(filename),
        }
    }

    /// Events were dropped: the client should reload its state.
    fn resync() -> Self {
        Self { kind: "resync", profile: None, download_url: None, filename: None }
    }
}

#[derive(Clone, Debug)]
struct Envelope {
    email: String,
    event: CvEvent,
}

pub struct EventBus {
    tx: broadcast::Sender<Envelope>,
}

impl Default for EventBus {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }
}

impl EventBus {
    /// Send `event` to every open stream of `email`. No-op when nobody listens.
    pub fn publish(&self, email: &str, event: CvEvent) {
        let _ = self.tx.send(Envelope { email: email.to_lowercase(), event });
    }

    /// Publish `event` when `result` is a success, and pass `result` through.
    pub fn on_ok<T, E>(&self, email: &str, result: Result<T, E>, event: impl FnOnce(&T) -> CvEvent) -> Result<T, E> {
        if let Ok(value) = &result {
            self.publish(email, event(value));
        }
        result
    }
}

/// GET /events — the authenticated user's change notifications, as SSE.
#[get("/events")]
pub fn events(auth: AuthenticatedUser, bus: &State<EventBus>, mut shutdown: Shutdown) -> EventStream![] {
    let mut rx = bus.tx.subscribe();
    let email = auth.email().to_lowercase();

    EventStream! {
        loop {
            let event = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) if msg.email == email => msg.event,
                    Ok(_) => continue,
                    Err(RecvError::Lagged(_)) => CvEvent::resync(),
                    Err(RecvError::Closed) => break,
                },
                _ = &mut shutdown => break,
            };
            yield Event::json(&event).event(event.kind);
        }
    }
    .heartbeat(HEARTBEAT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn on_ok_publishes_only_successes_under_the_lowercased_email() {
        let bus = EventBus::default();
        let mut rx = bus.tx.subscribe();

        let failed: Result<(), ()> = bus.on_ok("A@x.com", Err(()), |_| CvEvent::files_changed(None));
        assert!(failed.is_err());
        let ok: Result<u8, ()> = bus.on_ok("A@x.com", Ok(1), |_| CvEvent::files_changed(Some("p".into())));
        assert_eq!(ok, Ok(1));

        let msg = rx.recv().await.unwrap();
        assert_eq!(msg.email, "a@x.com");
        assert_eq!(msg.event.profile.as_deref(), Some("p"));
        assert!(rx.try_recv().is_err(), "the failed call must not publish");
    }
}
