use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = get_router();
    let tcp_listener = TcpListener::bind("0.0.0.0:3999").await.unwrap();

    axum::serve(tcp_listener, app).await.unwrap();
}

fn get_router() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/about", get(about_handler))
}

async fn root_handler() -> &'static str {
    "Hello world!"
}

async fn about_handler() -> &'static str {
    "Информационная страница о проекте"
}
