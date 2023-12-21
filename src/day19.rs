use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
};
use color_eyre::eyre::Report;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::broadcast::{self, Sender},
    task::JoinHandle,
};
use tracing::info;

use crate::ServerState;

pub async fn ready_game(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_serve_game)
}

async fn handle_serve_game(mut socket: WebSocket) {
    let mut game_started = false;
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };

        let msg_text = if let Ok(text) = msg.to_text() {
            text
        } else {
            return;
        };

        if msg_text == "serve" {
            game_started = true;
        } else if msg_text == "ping" && game_started {
            if socket.send("pong".into()).await.is_err() {
                // socket closed
                return;
            }
        } else {
            // ignore
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoomMessage {
    user: String,
    message: String,
}

#[derive(Debug, Clone, Default)]
pub struct BirdState {
    count: Arc<AtomicUsize>,
    room_users: Arc<Mutex<HashMap<usize, HashSet<String>>>>,
    room_broadcast: Arc<Mutex<HashMap<usize, Sender<RoomMessage>>>>,
}

pub async fn reset_tweet_count(State(state): State<ServerState>) -> impl IntoResponse {
    state.bird_state.count.store(0, Ordering::SeqCst);
}

pub async fn get_tweet_count(State(state): State<ServerState>) -> impl IntoResponse {
    state.bird_state.count.load(Ordering::SeqCst).to_string()
}

#[tracing::instrument(skip(state))]
pub async fn connect_room(
    Path((room_number, user_name)): Path<(usize, String)>,
    State(state): State<ServerState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let bird_state = state.bird_state;
    let tx = {
        let mut room_users = bird_state.room_users.lock().unwrap();
        if let Some(users) = room_users.get_mut(&room_number) {
            info!(room_number, %user_name, "Inserting new user to room");
            // room exists check name
            if users.contains(&user_name) {
                panic!("You already have the name!");
            } else {
                users.insert(user_name.clone());
                bird_state
                    .room_broadcast
                    .lock()
                    .unwrap()
                    .get(&room_number)
                    .unwrap()
                    .clone()
            }
        } else {
            // create a new room
            info!(room_number, %user_name, "Making a new room");
            let start_set = HashSet::from([user_name.clone()]);
            room_users.insert(room_number, start_set);
            let (tx, _) = broadcast::channel(100000);
            bird_state
                .room_broadcast
                .lock()
                .unwrap()
                .insert(room_number, tx.clone());
            tx.clone()
        }
    };
    ws.on_upgrade(|ws: WebSocket| async move {
        let count = bird_state.count.clone();
        handle_ws(user_name, count, tx, ws).await
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct UserMessage {
    message: String,
}

#[tracing::instrument(skip(ws))]
async fn handle_ws(
    user_name: String,
    count: Arc<AtomicUsize>,
    tx: Sender<RoomMessage>,
    ws: WebSocket,
) {
    let mut rx = tx.subscribe();
    let (mut sender, mut receiver) = ws.split();

    let mut msg_send_task: JoinHandle<Result<(), serde_json::Error>> = tokio::spawn(async move {
        while let Some(Ok(raw_msg)) = receiver.next().await {
            let msg = serde_json::from_str::<UserMessage>(raw_msg.to_text().unwrap())?;
            if msg.message.chars().count() > 128 {
                info!(?msg, ?user_name, "Message was too long.");
                continue;
            }
            info!(
                ?msg,
                ?user_name,
                "Got new Message from client, sending to room."
            );
            let send_msg = RoomMessage {
                user: user_name.clone(),
                message: msg.message,
            };
            let _ = tx.send(send_msg);
        }
        Ok(())
    });

    let mut msg_recv_task: JoinHandle<Result<(), Report>> = tokio::spawn(async move {
        while let Ok(new_msg) = rx.recv().await {
            info!(?new_msg, "Got a new message from the room.");
            sender
                .send(Message::Text(serde_json::to_string(&new_msg).unwrap()))
                .await?;
            // user has recieved the view increment it
            count.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    });

    tokio::select! {
        _ = (&mut msg_send_task) => {
            msg_recv_task.abort();
        }
        _ = (&mut msg_recv_task) => {
            msg_send_task.abort();
        }
    };
}
