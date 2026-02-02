use axum::{
    Json, Router,
    body::Body,
    extract::rejection::JsonRejection,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use validator::Validate;

#[tokio::main]
async fn main() {
    println!("Server starts...");

    let app = get_router();
    let tcp_listener = TcpListener::bind("0.0.0.0:3999").await.unwrap();

    axum::serve(tcp_listener, app).await.unwrap();
}

fn api_v1_routes() -> Router {
    Router::new()
        .route("/user", get(user_handler))
        .route("/session", post(log_in_handler))
        .fallback(|| async { StatusCode::NOT_FOUND }) // костыль для мидлваре
        // мидлваре
        .layer(middleware::from_fn(api_v1_error_interceptor))
}

async fn api_v1_error_interceptor(request: Request<Body>, next: Next) -> Response {
    let response = next.run(request).await;
    let status = response.status();

    if status == StatusCode::NOT_FOUND || status == StatusCode::METHOD_NOT_ALLOWED {
        println!("{}", status);
        let (code, message) = if status == StatusCode::NOT_FOUND {
            ("not_found", "Ресурс не найден")
        } else {
            ("method_not_allowed", "Метод не разрешен")
        };

        let body = serde_json::json!({
            "errors": [{
                "title": message,
                "detail": message,
                "code": code,
                "status": status.as_u16().to_string()
            }]
        });

        return (status, Json(body)).into_response();
    }

    response
}

fn get_router() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/about", get(about_handler))
        .nest("/api/v1", api_v1_routes())
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
