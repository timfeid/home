use crate::error::{AppError, AppResult};
use crate::message::{IncomingMessage, OutgoingMessage};
use crate::state::{AppState, ClientInfo, ClientSender};
use futures::{SinkExt, StreamExt, TryStreamExt};
use std::sync::Arc;
use talky_auth::JwtService;
use tokio::sync::Mutex;
use ulid::Ulid;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

pub async fn handle_connection(ws: WebSocket, state: AppState) {
    tracing::info!("New WebSocket connection established");

    let (sender, receiver) = ws.split();
    let sender = Arc::new(Mutex::new(sender));

    let mut receiver = receiver.map_err(AppError::WebSocket);

    let (client_id, join_code, initial_client_info) =
        match handle_initialization(&mut receiver, sender.clone(), state.clone()).await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!("Initialization failed: {:?}", e);

                let close_msg = e.to_ws_close_message();
                let _ = sender.lock().await.send(close_msg).await;

                return;
            }
        };

    tracing::info!("Client {} initialized for room '{}'", client_id, join_code);

    if let Err(e) = state
        .add_client_to_room(join_code.clone(), initial_client_info)
        .await
    {
        tracing::error!(
            "Failed to add client {} to room {}: {:?}",
            client_id,
            join_code,
            e
        );

        let err_msg = OutgoingMessage::Error {
            message: format!("Failed to join room: {:?}", e),
        };
        let _ = sender
            .lock()
            .await
            .send(
                err_msg
                    .to_ws_message()
                    .unwrap_or(Message::text("Error joining room")),
            )
            .await;
        let _ = sender.lock().await.close().await;
        return;
    }

    if let Err(e) = message_loop(
        receiver,
        sender.clone(),
        state.clone(),
        &join_code,
        &client_id,
    )
    .await
    {
        tracing::error!(
            "Error during message loop for client {}: {:?}",
            client_id,
            e
        );
    }

    tracing::info!(
        "Client {} disconnected from room '{}'",
        client_id,
        join_code
    );
    state.remove_client_from_room(&join_code, &client_id).await;
}

async fn handle_initialization(
    receiver: &mut (impl StreamExt<Item = AppResult<Message>> + Unpin),
    sender: ClientSender,
    state: AppState,
) -> AppResult<(String, String, ClientInfo)> {
    let init_msg = receiver
        .next()
        .await
        .ok_or(AppError::ClientDisconnected)??;

    if !init_msg.is_text() {
        tracing::warn!("Received non-text initial message");
        return Err(AppError::InitializationError(
            "Initial message must be text".to_string(),
        ));
    }

    let init_text = init_msg.to_str().map_err(|_| {
        AppError::InitializationError("Invalid UTF-8 in initial message".to_string())
    })?;

    let init_data: IncomingMessage = serde_json::from_str(init_text).map_err(AppError::Json)?;

    let (join_code, auth_code, role) = match init_data {
        IncomingMessage::Init {
            join_code,
            auth_code,
            role,
        } => {
            if join_code.is_empty() || auth_code.is_empty() || role.is_empty() {
                return Err(AppError::InitializationError(
                    "join_code, auth_code, and role cannot be empty".to_string(),
                ));
            }
            (join_code, auth_code, role)
        }
        _ => {
            return Err(AppError::InitializationError(
                "First message must be of type 'init'".to_string(),
            ));
        }
    };

    let token_data = JwtService::decode(&auth_code)?;
    tracing::debug!(
        "Client authenticated successfully: User ID {}",
        token_data.claims.sub
    );

    let client_id = Ulid::new().to_string();
    let client_info = ClientInfo {
        id: client_id.clone(),
        user_id: token_data.claims.sub,
        role,
        sender,
    };

    Ok((client_id, join_code, client_info))
}

async fn message_loop(
    mut receiver: impl StreamExt<Item = AppResult<Message>> + Unpin,
    sender: ClientSender,
    state: AppState,
    join_code: &str,
    client_id: &str,
) -> AppResult<()> {
    let role = {
        let rooms_guard = state.rooms.lock().await;
        rooms_guard
            .get(join_code)
            .and_then(|r| r.clients.get(client_id).map(|c| c.role.clone()))
            .unwrap_or_else(|| "unknown".to_string())
    };

    while let Some(message_result) = receiver.next().await {
        match message_result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap_or("");
                    match handle_text_message(text, &state, join_code, client_id, &role).await {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::warn!(
                                "Error processing message from client {}: {:?}",
                                client_id,
                                e
                            );

                            let err_msg = OutgoingMessage::Error {
                                message: format!("Error processing message: {:?}", e),
                            };
                            let _ = sender
                                .lock()
                                .await
                                .send(
                                    err_msg
                                        .to_ws_message()
                                        .unwrap_or(Message::text("Processing error")),
                                )
                                .await;
                        }
                    }
                } else if msg.is_binary() {
                    tracing::debug!("Received binary message from {}", client_id);
                } else if msg.is_ping() {
                    tracing::trace!("Received Ping from {}", client_id);

                    if let Err(e) = sender
                        .lock()
                        .await
                        .send(Message::pong(msg.into_bytes()))
                        .await
                    {
                        tracing::warn!("Failed to send Pong to {}: {}", client_id, e);

                        return Err(AppError::ClientSendError);
                    }
                } else if msg.is_pong() {
                    tracing::trace!("Received Pong from {}", client_id);
                } else if msg.is_close() {
                    tracing::info!("Received close frame from client {}", client_id);

                    return Ok(());
                }
            }
            Err(e) => {
                tracing::error!("WebSocket read error for client {}: {:?}", client_id, e);
                return Err(e);
            }
        }
    }

    tracing::info!("Client {} connection stream ended.", client_id);
    Ok(())
}

async fn handle_text_message(
    text: &str,
    state: &AppState,
    join_code: &str,
    client_id: &str,
    client_role: &str,
) -> AppResult<()> {
    match serde_json::from_str::<IncomingMessage>(text) {
        Ok(incoming_msg) => {
            tracing::debug!("Received message from {}: {:?}", client_id, incoming_msg);
            match incoming_msg {
                IncomingMessage::Init { .. } => {
                    tracing::warn!(
                        "Client {} sent unexpected 'init' message after initialization.",
                        client_id
                    );
                }
                IncomingMessage::ChatMessage { content } => {
                    state
                        .handle_chat_message(join_code, client_id, client_role, content)
                        .await;
                }
                IncomingMessage::WebRtcSignal {
                    target_client_id,
                    signal_data,
                } => {
                    state
                        .handle_webrtc_signal(join_code, client_id, &target_client_id, signal_data)
                        .await?;
                }
            }
            Ok(())
        }
        Err(e) => {
            tracing::warn!(
                "Failed to parse message from client {}: {}, message: '{}'",
                client_id,
                e,
                text
            );
            Err(AppError::Json(e))
        }
    }
}
