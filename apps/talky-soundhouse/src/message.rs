use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingMessage {
    Init {
        auth_code: String,
    },
    // ChatMessage {
    //     content: String,
    // },
    WebRtcSignal {
        target_client_id: String,
        signal_data: Value,
    },
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingMessage {
    ActiveClientsUpdate {
        join_code: String,
        clients: Vec<ClientInfoMsg>,
    },

    ChatMessageBroadcast {
        sender_id: String,
        sender_role: String,
        content: String,
    },

    WebRtcSignal {
        sender_client_id: String,
        signal_data: Value,
    },
    Error {
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfoMsg {
    pub id: String,
    pub role: String,
}

impl OutgoingMessage {
    pub fn to_ws_message(&self) -> Result<warp::ws::Message, serde_json::Error> {
        let json_string = serde_json::to_string(self)?;
        Ok(warp::ws::Message::text(json_string))
    }
}
