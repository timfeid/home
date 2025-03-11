use std::{fs::write, future::IntoFuture, path::PathBuf, sync::Arc};

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        request::Parts,
        Method,
    },
    routing::get,
};
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
use tokio::sync::Mutex;

use rspc::Router;
use tower_http::cors::{AllowOrigin, CorsLayer};

async fn create_pool() -> Arc<Pool<Postgres>> {
    let database_url = dotenv::var("DATABASE_URL").unwrap();
    create_connection(&database_url).await
}

async fn create_lobby_manager() -> Arc<LobbyManager> {
    let manager = LobbyManager::new().await.unwrap();
    Arc::new(manager)
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

    axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        .nest(
            "/rspc",
            rspc_axum::endpoint(procedures, move |parts: Parts| {
                Ctx::new(pool.clone(), parts, lobby_manager.clone())
            }),
        )
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
