use std::str;

use axum::{
    Json, Router,
    extract::rejection::{self, JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use validator::Validate;

#[tokio::main]
async fn main() {
    println!("Server starts...");

    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "onlyweb=debug,tower_http=debug".into()),
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    let app = get_router();
    let tcp_listener = TcpListener::bind("0.0.0.0:3999").await.unwrap();

    axum::serve(tcp_listener, app).await.unwrap();
}

fn api_v1_routes() -> Router {
    Router::new()
        .route("/user", get(user_handler))
        .route("/session", post(log_in_handler))
}

fn get_router() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/about", get(about_handler))
        .route("/api/v1/user", get(user_handler))
        .route("/api/v1/session", post(log_in_handler))
    // .nest("/api/v1", api_v1_routes())
    // .layer(
    //     TraceLayer::new_for_http()
    //         .make_span_with(|req: &Request| {
    //             let method = req.method();
    //             let uri = req.uri();
    //             let match_path = req
    //                 .extentions()
    //                 .get::<MatchedPath>()
    //                 .map(|match_path| match_path.as_str());
    //             tracing::debug_span!("request", %method, %uri, match_path)
    //         })
    //         .on_failure(()),
    // )
    // .with_state(1)
}

async fn root_handler() -> &'static str {
    "Привет, мир!"
}

async fn about_handler() -> &'static str {
    "Информационная страница о проекте"
}

async fn user_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "meta": {} }))
}

async fn log_in_handler(payload: Result<Json<LogInAttributes>, JsonRejection>) -> Response {
    let Json(body) = match payload {
        Ok(data) => data,
        Err(e) => {
            return (
                e.status(),
                Json(serde_json::json!({"errors": [{"title": "", "detail": e.body_text(), "status": e.status().as_u16().to_string()}]})),
            )
                .into_response();
        }
    };
    if let Err(errors) = body.validate() {
        // Собираем все ошибки в твой формат
        let messages: Vec<_> = errors
            .field_errors()
            .iter()
            .map(|(field, errs)| ErrorDetail {
                source: Source {
                    pointer: Some(format!("/data/attributes/{}", field)),
                    header: None,
                    parameter: None,
                },
                title: "".to_string(),
                detail: errs[0]
                    .message
                    .as_ref()
                    .unwrap_or(&"Invalid value".into())
                    .to_string(),
                status: "".to_string(),
            })
            .collect();

        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorsResponse { errors: messages }),
        )
            .into_response();
    }

    // match payload {
    //     Ok(Json(body)) => {
    //         // Твоя логика...
    //         println!("User: {}", body.username);
    //
    //         (StatusCode::CREATED, Json(serde_json::json!({ "meta": {}}))).into_response()
    //     }
    //     Err(rejection) => {
    //         // Превращаем ошибку Axum в нужный формат { errors: [{message: "..."}] }
    //         let error_json = ErrorsResponse {
    //             errors: vec![ErrorDetail {
    //                 title: "".to_string(),         //rejection.status().canonical_reason(),
    //                 detail: rejection.body_text(), // или кастомный текст
    //                 status: rejection.status().as_u16().to_string(),
    //             }],
    //         };
    //
    //         (rejection.status(), Json(error_json)).into_response()
    //     }
    // }
    (StatusCode::OK, Json(serde_json::json!({"meta": {}}))).into_response()
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct LogInAttributes {
    #[validate(required(message = "Логин не может быть пустым"))]
    #[validate(length(
        min = 1,
        message = "Логин недостаточной длины (не может быть меньше 1 символов)"
    ))]
    // #[validate(length(
    //     max = 64,
    //     message = "Логин слишком большой длины (не может быть больше чем 64 символ)"
    // ))]
    username: Option<String>,
    #[validate(required(message = "Пароль не может быть пустым"))]
    #[validate(length(
        min = 10,
        message = "Пароль недостаточной длины (не может быть меньше 10 символов)"
    ))]
    // #[validate(length(
    //     max = 256,
    //     message = "Пароль слишком большой длины (не может быть больше чем 256 символ)"
    // ))]
    password: Option<String>,
}

// Основная структура ошибки
#[derive(Serialize)]
struct ErrorsResponse {
    errors: Vec<ErrorDetail>,
}

#[derive(Serialize)]
struct ErrorDetail {
    source: Source,
    title: String,
    detail: String,
    status: String,
}

#[derive(Serialize)]
struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameter: Option<String>,
}

// Вспомогательная структура для успешного ответа
#[derive(Serialize)]
struct SuccessResponse {
    meta: std::collections::HashMap<String, String>,
}

// #[derive(Serialize)]
// struct MetaResponse {
//     meta: ,
// }
