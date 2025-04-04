use crate::error::{AppError, AppResult};
use crate::message::{IncomingMessage, OutgoingMessage};
use crate::state::{AppState, ClientInfo, ClientSender};
use futures::{SinkExt, StreamExt, TryStreamExt};
use std::sync::Arc;
use talky_auth::JwtService;
use tokio::sync::Mutex;
use ulid::Ulid;
use warp::ws::{Message, WebSocket};

pub async fn handle_connection(ws: WebSocket, state: AppState) {
    tracing::info!("New WebSocket connection established");

    let (sender, receiver) = ws.split();
    let sender = Arc::new(Mutex::new(sender));
    let mut receiver = receiver.map_err(AppError::WebSocket);

    if let Err(e) = initialize_client(&mut receiver, sender.clone(), state.clone()).await {
        tracing::error!("Initialization error: {:?}", e);
        let close_msg = e.to_ws_close_message();
        let _ = sender.lock().await.send(close_msg).await;
        return;
    }
}

async fn initialize_client(
    receiver: &mut (impl StreamExt<Item = AppResult<Message>> + Unpin),
    sender: ClientSender,
    state: AppState,
) -> AppResult<()> {
    let (client_id, initial_client_info) =
        validate_initialization(receiver, sender.clone()).await?;

    if let Err(e) = state.add_client(initial_client_info).await {
        handle_client_error(sender, client_id.clone(), e).await;
        return Ok(());
    }

    if let Err(e) = handle_messages(receiver, sender.clone(), state.clone(), &client_id).await {
        tracing::error!(
            "Error during message loop for client {}: {:?}",
            client_id,
            e
        );
    }

    tracing::info!("Client {} disconnected", client_id);
    state.remove_client(&client_id).await;
    Ok(())
}

async fn validate_initialization(
    receiver: &mut (impl StreamExt<Item = AppResult<Message>> + Unpin),
    sender: ClientSender,
) -> AppResult<(String, ClientInfo)> {
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
    let auth_code = parse_init_data(init_data)?;

    let token_data = JwtService::decode(&auth_code)?;
    tracing::debug!(
        "Client authenticated successfully: User ID {}",
        token_data.claims.sub
    );

    let client_id = Ulid::new().to_string();
    let client_info = ClientInfo {
        id: client_id.clone(),
        user_id: token_data.claims.sub,
        role: "presence".to_string(),
        sender,
    };

    Ok((client_id, client_info))
}

fn parse_init_data(init_data: IncomingMessage) -> AppResult<String> {
    match init_data {
        IncomingMessage::Init { auth_code } => {
            if auth_code.is_empty() {
                return Err(AppError::InitializationError(
                    "auth_code cannot be empty".to_string(),
                ));
            }
            Ok(auth_code)
        }
        _ => Err(AppError::InitializationError(
            "First message must be of type 'init'".to_string(),
        )),
    }
}

async fn handle_client_error(sender: ClientSender, client_id: String, error: AppError) {
    tracing::error!("Failed to add client {} : {:?}", client_id, error);

    let err_msg = OutgoingMessage::Error {
        message: format!("Failed to join room: {:?}", error),
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
}

async fn handle_messages(
    mut receiver: impl StreamExt<Item = AppResult<Message>> + Unpin,
    sender: ClientSender,
    state: AppState,
    client_id: &str,
) -> AppResult<()> {
    let role = {
        let rooms_guard = state.rooms.lock().await;
        rooms_guard
            .values()
            .find_map(|r| r.clients.get(client_id).map(|c| c.role.clone()))
            .unwrap_or_else(|| "unknown".to_string())
    };

    while let Some(message_result) = receiver.next().await {
        process_message(message_result, sender.clone(), &state, client_id, &role).await?;
    }

    tracing::info!("Client {} connection stream ended.", client_id);
    Ok(())
}

async fn process_message(
    message_result: AppResult<Message>,
    sender: ClientSender,
    state: &AppState,
    client_id: &str,
    role: &str,
) -> AppResult<()> {
    match message_result {
        Ok(msg) if msg.is_text() => {
            let text = msg.to_str().unwrap_or("");
            if let Err(e) = handle_text_message(text, state, client_id, role).await {
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
        Ok(msg) => process_non_text_message(msg, sender, client_id).await?,
        Err(e) => {
            tracing::error!("WebSocket read error for client {}: {:?}", client_id, e);
            return Err(e);
        }
    }
    Ok(())
}

async fn process_non_text_message(
    msg: Message,
    sender: ClientSender,
    client_id: &str,
) -> AppResult<()> {
    if msg.is_binary() {
        tracing::debug!("Received binary message from {}", client_id);
    } else if msg.is_ping() {
        tracing::trace!("Received Ping from {}", client_id);
        sender
            .lock()
            .await
            .send(Message::pong(msg.into_bytes()))
            .await
            .map_err(|e| {
                tracing::warn!("Failed to send Pong to {}: {}", client_id, e);
                AppError::ClientSendError
            })?;
    } else if msg.is_pong() {
        tracing::trace!("Received Pong from {}", client_id);
    } else if msg.is_close() {
        tracing::info!("Received close frame from client {}", client_id);
    }
    Ok(())
}

async fn handle_text_message(
    text: &str,
    state: &AppState,
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
                IncomingMessage::WebRtcSignal {
                    target_client_id,
                    signal_data,
                } => {
                    state
                        .handle_webrtc_signal(client_id, &target_client_id, signal_data)
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
