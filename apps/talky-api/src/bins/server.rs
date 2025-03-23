use std::{fs::write, future::IntoFuture, path::PathBuf, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        request::Parts,
        Method,
    },
    response::IntoResponse,
    routing::get,
    Extension,
};
use futures::{SinkExt, StreamExt};
use rusty::{
    database::create_connection,
    http::{context::Ctx, routers::mount},
    lobby::manager::LobbyManager,
};
// use database::create_connection;
// use error::{AppError, AppResult};
// use http::routers::create_router;
// use lobby::manager::LobbyManager;
// use services::jwt::{Claims, JwtService};
use sqlx::{Executor, Pool, Postgres};
use tokio::sync::{broadcast, Mutex};

use rspc::Router;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Clone)]
struct AppState {
    broadcaster: broadcast::Sender<Message>,
}

async fn create_pool() -> Arc<Pool<Postgres>> {
    let database_url = dotenv::var("DATABASE_URL").unwrap();
    create_connection(&database_url).await
}

async fn create_lobby_manager() -> Arc<LobbyManager> {
    let manager = LobbyManager::new().await.unwrap();
    Arc::new(manager)
}

async fn ws_handler(
    Extension(state): Extension<AppState>, // extract shared state
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Split the socket so we can concurrently read from it and write to it.
    let (mut sender, mut receiver) = socket.split();
    // Subscribe to the broadcast channel.
    let mut rx = state.broadcaster.subscribe();

    // Spawn a task to forward broadcast messages to this socket.
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = sender.send(msg).await {
                eprintln!("Error sending broadcast message: {:?}", e);
                break;
            }
        }
    });

    // Process incoming messages from the client.
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                // Broadcast the received text message to all clients.
                if let Err(e) = state.broadcaster.send(Message::Text(text.clone())) {
                    eprintln!("Error broadcasting text message: {:?}", e);
                }
            }
            Ok(Message::Binary(bin)) => {
                // Broadcast the binary message.
                if let Err(e) = state.broadcaster.send(Message::Binary(bin)) {
                    eprintln!("Error broadcasting binary message: {:?}", e);
                }
            }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }

    // Close the sender task when the connection is terminated.
    send_task.abort();
}

async fn create_app() -> axum::Router {
    let router = mount();
    let (procedures, types) = router.build().unwrap();

    rspc::Typescript::default()
        // .formatter(specta_typescript::formatter::prettier)
        .header("// My custom header")
        // .enable_source_maps() // TODO: Fix this
        .export_to(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"),
            &types,
        )
        .unwrap();

    let allowed_headers = [CONTENT_TYPE, AUTHORIZATION];
    let allowed_methods = [Method::GET, Method::POST, Method::OPTIONS];
    let pool = create_pool().await;
    let lobby_manager = create_lobby_manager().await;
    let (broadcaster, _) = broadcast::channel::<Message>(100);

    let app_state = AppState { broadcaster };

    axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            rspc_axum::endpoint(procedures, move |parts: Parts| {
                Ctx::new(pool.clone(), parts, lobby_manager.clone())
            }),
        )
        .route("/ws", get(ws_handler))
        .layer(Extension(app_state))
        .layer(
            CorsLayer::new()
                .allow_methods(allowed_methods)
                .allow_headers(allowed_headers)
                .allow_origin(AllowOrigin::mirror_request())
                .allow_credentials(true),
        )
}

// async fn handler(context: Ctx) {
//     let account = Account::find(&context.pool.clone(), "test".to_string())
//         .await
//         .expect("hi");

//     let totp = &account.get_current_code().expect("hi");
//     let token = totp.generate_current().unwrap();

//     println!("{:?}, token: {}", account, token);
// }

#[tokio::main]
async fn main() {
    // handler(context).await;

    let app = create_app().await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
