use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{map::Values, Value};
use specta::Type;
use talky_services::message::service::MessageResource;

use crate::state::{RoomClientInfo, RoomResource};

#[derive(Type, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[specta(rename = "OutgoingMessage")]
pub enum IncomingMessage {
    Init {
        auth_code: String,
    },
    UpdateNiche {
        niche_id: String,
    },
    Join {
        channel_id: String,
        role: String,
    },
    Candidate {
        candidate: Value,

        channel_id: String,
        // todo remove this
        niche_id: String,
    },
    Answer {
        answer: String,
        channel_id: String,
        // todo remove this
        niche_id: String,
    },
    Offer {
        offer: String,
        channel_id: String,
        // todo remove this
        niche_id: String,
    },
    ChatMessage {
        content: String,
        channel_id: String,
    },
    WebRtcSignal {
        target_client_id: String,
        signal_data: Value,
    },
}

#[derive(Type, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[specta(rename = "IncomingMessage")]
pub enum OutgoingMessage {
    ActiveChannels {
        channels: HashMap<String, RoomResource>,
    },

    Candidate {
        candidate: Value,
    },

    Answer {
        answer: String,
    },

    Offer {
        offer: String,
    },

    ActiveClientsUpdate {
        clients: Vec<ClientInfoMsg>,
    },

    ChatMessageBroadcast {
        sender_id: String,
        message: MessageResource,
        channel_id: String,
    },

    WebRtcSignal {
        sender_client_id: String,
        signal_data: Value,
    },
    Error {
        message: String,
    },
}

#[derive(Type, Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfoMsg {
    pub user_id: String,
}

impl OutgoingMessage {
    pub fn to_ws_message(&self) -> Result<warp::ws::Message, serde_json::Error> {
        let json_string = serde_json::to_string(self)?;
        Ok(warp::ws::Message::text(json_string))
    }
}
