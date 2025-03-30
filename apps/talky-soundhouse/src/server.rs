use crate::handler::handle_connection;
use crate::state::AppState;
use std::convert::Infallible;
use warp::Filter;

fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

pub fn build_routes(
    state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let soundhouse_route = warp::path("soundhouse")
        .and(warp::ws())
        .and(with_state(state))
        .map(|ws: warp::ws::Ws, state: AppState| {
            ws.on_upgrade(move |socket| handle_connection(socket, state))
        });

    // let health_route = warp::path("health").map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    soundhouse_route // .or(health_route)
}

/*
use crate::error::AppError;
use warp::http::StatusCode;
use warp::reply::with_status;

pub async fn handle_rejection(rej: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if rej.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if let Some(e) = rej.find::<AppError>() {

        match e {
            AppError::JwtAuth(_) => {
                code = StatusCode::UNAUTHORIZED;
                message = "Authentication failed".to_string();
            }
            _ => {
                tracing::error!("Unhandled rejection: {:?}", e);
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = "INTERNAL_SERVER_ERROR".to_string();
            }
        }
    } else if rej.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_string();
    } else {

        tracing::error!("Unhandled warp rejection: {:?}", rej);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR".to_string();
    }

    let json = warp::reply::json(&serde_json::json!({
        "code": code.as_u16(),
        "message": message,
    }));

    Ok(with_status(json, code))
}
*/
