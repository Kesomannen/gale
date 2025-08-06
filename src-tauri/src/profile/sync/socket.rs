use eyre::Result;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_websockets::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::{
    profile::{ModManager, Profile},
    state::ManagerExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "event", content = "payload", rename_all = "camelCase")]
enum ServerMessage {
    #[serde(rename_all = "camelCase")]
    ProfileUpdated {
        metadata: super::SyncProfileMetadata,
    },

    #[serde(rename_all = "camelCase")]
    ProfileDeleted {
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

type WebSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

pub struct State {
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

        Self { tx }
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

async fn connect(app: AppHandle, rx: mpsc::UnboundedReceiver<ClientMessage>) -> Result<()> {
    let url = format!("{}/socket/connect", super::API_URL.replace("http", "ws"));

    info!("connecting to sync server via socket at {url}");

    let (socket, response) = tokio_websockets::ClientBuilder::new()
        .uri(&url)
        .unwrap()
        .connect()
        .await?;

    info!("connected to sync server via socket: {response:?}");

    let (sender, receiver) = socket.split();

    tokio::spawn(read(app.to_owned(), receiver));
    tokio::spawn(write(sender, rx));

    Ok(())
}

async fn read(app: AppHandle, mut receiver: SplitStream<WebSocket>) {
    while let Some(item) = receiver.next().await {
        let item = match item {
            Ok(item) => item,
            Err(err) => {
                error!("socket error: {err}");
                return;
            }
        };

        let Some(text) = item.as_text() else {
            warn!("got non-text message from socket");
            continue;
        };

        let msg = match serde_json::from_str::<ServerMessage>(text) {
            Ok(msg) => msg,
            Err(err) => {
                warn!("failed to deserialize server message: {err}");
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

                    profile.save(app.db()).ok();
                }

                app.emit("profiles_changed", ()).ok();
            }
            ServerMessage::ProfileDeleted { id } => {
                info!("got sync profile delete event for {}", id);

                let mut manager = app.lock_manager();

                for profile in sync_profiles_with_id(&mut manager, &id) {
                    profile.sync = None;

                    profile.save(app.db()).ok();
                }

                app.emit("profiles_changed", ()).ok();
            }
            ServerMessage::Error { message } => {
                error!("got error from socket: {message}");
            }
        }
    }
}

async fn write(
    mut sender: SplitSink<WebSocket, tokio_websockets::Message>,
    mut rx: mpsc::UnboundedReceiver<ClientMessage>,
) {
    while let Some(msg) = rx.recv().await {
        debug!("sending {msg:?}");

        let msg = match serde_json::to_string(&msg) {
            Ok(str) => tokio_websockets::Message::text(str),
            Err(err) => {
                error!("failed to serialize socket message: {err}");
                continue;
            }
        };

        if let Err(err) = sender.send(msg).await {
            warn!("stopping socket write task: transmit error: {err}");
            return;
        }
    }

    info!("stopping socket write task: channel was closed")
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
