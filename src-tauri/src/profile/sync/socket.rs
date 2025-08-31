use std::sync::atomic::{AtomicBool, Ordering};

use eyre::Result;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use reqwest_websocket::{RequestBuilderExt, WebSocket};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    profile::{ModManager, Profile},
    state::ManagerExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "event", content = "payload", rename_all = "camelCase")]
enum ServerMessage {
    ProfileUpdated {
        metadata: super::SyncProfileMetadata,
    },

    ProfileDeleted {
        id: String,
    },

    ProfileNotFound {
        id: String,
    },

    Error {
        message: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "payload", rename_all = "camelCase")]
enum ClientMessage {
    #[serde(rename_all = "camelCase")]
    Subscribe { profile_id: String },

    #[serde(rename_all = "camelCase")]
    Unsubscribe { profile_id: String },
}

pub struct State {
    connected: AtomicBool,
    tx: mpsc::UnboundedSender<ClientMessage>,
}

impl State {
    pub fn new(app: AppHandle) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        tauri::async_runtime::spawn(async move {
            if let Err(err) = connect(app, rx).await {
                error!("failed to connect to sync server: {err:#}");
            }
        });

        Self {
            connected: AtomicBool::new(false),
            tx,
        }
    }

    pub fn subscribe(&self, profile: &Profile) {
        let Some(info) = profile.sync.as_ref() else {
            return;
        };

        self.send(ClientMessage::Subscribe {
            profile_id: info.id.clone(),
        });
    }

    pub fn unsubscribe(&self, profile: &Profile) {
        let Some(info) = profile.sync.as_ref() else {
            return;
        };

        self.send(ClientMessage::Unsubscribe {
            profile_id: info.id.clone(),
        });
    }

    fn send(&self, msg: ClientMessage) {
        debug!("queueing {msg:?}");

        if self.tx.send(msg).is_err() {
            error!("failed to queue message to socket task");
        }
    }
}

async fn connect(app: AppHandle, mut rx: mpsc::UnboundedReceiver<ClientMessage>) -> Result<()> {
    // wait until we want to send a message before connecting
    let Some(first) = rx.recv().await else {
        return Ok(()); // channel was closed before first message
    };

    let url = format!("{}/socket/connect", super::API_URL.replace("http", "ws"));

    info!("connecting to sync server socket at {url}");

    let socket = app
        .http()
        .get(&url)
        .upgrade()
        .send()
        .await?
        .into_websocket()
        .await?;

    let (mut sender, receiver) = socket.split();

    send_queued_message(&mut sender, first).await;
    tokio::spawn(write(sender, rx));

    app.sync_socket().connected.store(true, Ordering::Relaxed);

    read(&app, receiver).await;

    app.sync_socket().connected.store(false, Ordering::Relaxed);

    Ok(())
}

async fn read(app: &AppHandle, mut receiver: SplitStream<WebSocket>) {
    while let Some(item) = receiver.next().await {
        let item = match item {
            Ok(item) => item,
            Err(err) => {
                error!("socket error, aborting task: {err}");
                return;
            }
        };

        let msg: ServerMessage = match item.json() {
            Ok(msg) => msg,
            Err(err) => {
                error!("failed to deserialize message: {err}");
                continue;
            }
        };

        match msg {
            ServerMessage::ProfileUpdated { metadata } => {
                info!("got sync profile update event for {}", metadata.id);

                let mut manager = app.lock_manager();

                for profile in sync_profiles_with_id(&mut manager, &metadata.id) {
                    let info = profile.sync.as_mut().unwrap();
                    info.updated_at = metadata.updated_at;
                    info.owner = metadata.owner.clone();

                    profile.save(&app, true).ok();
                }
            }
            ServerMessage::ProfileNotFound { id } | ServerMessage::ProfileDeleted { id } => {
                info!("got sync profile delete event for {}", id);

                let mut manager = app.lock_manager();

                for profile in sync_profiles_with_id(&mut manager, &id) {
                    profile.sync.as_mut().unwrap().missing = true;

                    profile.save(&app, true).ok();
                }
            }
            ServerMessage::Error { message } => {
                error!("got error from socket: {message}");
            }
        }
    }
}

async fn write(
    mut sender: SplitSink<WebSocket, reqwest_websocket::Message>,
    mut rx: mpsc::UnboundedReceiver<ClientMessage>,
) {
    while let Some(msg) = rx.recv().await {
        send_queued_message(&mut sender, msg).await;
    }

    info!("stopping socket write task: channel was closed")
}

async fn send_queued_message(
    sender: &mut SplitSink<WebSocket, reqwest_websocket::Message>,
    msg: ClientMessage,
) {
    debug!("sending {msg:?}");

    let msg = match serde_json::to_string(&msg) {
        Ok(str) => reqwest_websocket::Message::Text(str),
        Err(err) => {
            warn!("failed to serialize socket message: {err}");
            return;
        }
    };

    if let Err(err) = sender.send(msg).await {
        warn!("failed to send socket message: {err}");
    }
}

fn sync_profiles_with_id<'a>(
    manager: &'a mut ModManager,
    id: &'a str,
) -> impl Iterator<Item = &'a mut Profile> + 'a {
    manager
        .games
        .values_mut()
        .flat_map(|game| game.profiles.iter_mut())
        .filter(move |profile| profile.sync.as_ref().is_some_and(|info| info.id == id))
}
