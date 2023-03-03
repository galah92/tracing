use axum::{
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::{DefaultMakeSpan, TraceLayer},
    ServiceBuilderExt,
};
use tracing::Level;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("hyper=warn".parse().unwrap()); // disable hyper's debug logging
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // based on https://docs.rs/tower-http/0.2.5/tower_http/request_id/index.html#using-trace
    let trace_layer = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http().make_span_with(
                DefaultMakeSpan::new()
                    .include_headers(true)
                    .level(Level::INFO),
            ),
        )
        .propagate_x_request_id();

    let app = Router::new()
        .route("/", get(hello))
        .route("/users", post(create_user))
        .layer(trace_layer);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tracing::instrument]
async fn hello() -> &'static str {
    "Hello, World!"
}

#[derive(Clone)]
struct MakeRequestUuid;

// based on https://docs.rs/tower-http/0.2.5/tower_http/request_id/index.html#using-uuids
impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();
        Some(RequestId::new(request_id.parse().unwrap()))
    }
}

#[tracing::instrument]
async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    tracing::info!("creating user");
    let user = User {
        id: 1,
        username: payload.username,
    };
    (StatusCode::CREATED, Json(user))
}

#[derive(Deserialize, Debug)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: i64,
    username: String,
}
