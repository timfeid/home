// modules/signaling.rs
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

pub type WebSocketSender = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

pub async fn connect_to_signaling_server(
    url: &str,
) -> Result<(
    WebSocketSender,
    futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
)> {
    let (ws_stream, _) = tokio::time::timeout(Duration::from_secs(10), connect_async(url))
        .await
        .context("WebSocket connection timeout")?
        .context("Failed to connect to signaling server")?;

    let (ws_sender, ws_receiver) = ws_stream.split();
    Ok((ws_sender, ws_receiver))
}

pub async fn send_init_message(ws_sender: &mut WebSocketSender, auth_code: &str) -> Result<()> {
    let init_msg = json!({
        "type": "init",
        "auth_code": auth_code,
    })
    .to_string();

    ws_sender
        .send(Message::Text(init_msg.into()))
        .await
        .context("Failed to send init message")?;

    Ok(())
}

pub async fn send_join_message(ws_sender: &mut WebSocketSender, channel_id: &str) -> Result<()> {
    let msg = json!({
        "type": "join",
        "channel_id": channel_id,
        "role": "offerer"
    })
    .to_string();

    ws_sender
        .send(Message::Text(msg.into()))
        .await
        .context("Failed to send join message")?;

    Ok(())
}

pub async fn send_offer(
    ws_sender: &mut WebSocketSender,
    sdp: &str,
    channel_id: &str,
    niche_id: &str,
) -> Result<()> {
    let offer_msg = json!({
        "type": "offer",
        "niche_id": niche_id,
        "offer": sdp,
        "channel_id": channel_id
    })
    .to_string();

    ws_sender
        .send(Message::Text(offer_msg.into()))
        .await
        .context("Failed to send offer message")?;

    Ok(())
}

pub async fn process_candidate_message(
    peer_connection: &RTCPeerConnection,
    candidate_obj: &Value,
) -> Result<()> {
    let candidate_json = serde_json::to_string(candidate_obj)?;
    let candidate_init: RTCIceCandidateInit = serde_json::from_str(&candidate_json)?;

    peer_connection
        .add_ice_candidate(candidate_init)
        .await
        .context("Failed to add ICE candidate")?;

    Ok(())
}

pub async fn process_answer_message(
    peer_connection: &RTCPeerConnection,
    answer_sdp: &str,
) -> Result<()> {
    let answer = RTCSessionDescription::answer(answer_sdp.to_string())?;

    peer_connection
        .set_remote_description(answer)
        .await
        .context("Failed to set remote description")?;

    Ok(())
}
