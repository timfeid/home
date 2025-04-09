use lib::{
    message::{ClientInfoMsg, IncomingMessage, OutgoingMessage},
    state::{RoomResource, UserResource, UserRoomResource},
};
use serde_json::Value;
use talky_services::message::service::MessageResource;

fn main() {
    std::fs::write(
        "./types.d.ts",
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            specta_typescript::export::<MessageResource>(&Default::default()).unwrap(),
            specta_typescript::export::<UserResource>(&Default::default()).unwrap(),
            specta_typescript::export::<UserRoomResource>(&Default::default()).unwrap(),
            specta_typescript::export::<ClientInfoMsg>(&Default::default()).unwrap(),
            specta_typescript::export::<Value>(&Default::default()).unwrap(),
            specta_typescript::export::<RoomResource>(&Default::default()).unwrap(),
            specta_typescript::export::<IncomingMessage>(&Default::default()).unwrap(),
            specta_typescript::export::<OutgoingMessage>(&Default::default()).unwrap()
        ),
    )
    .expect("something went wrong");
}
