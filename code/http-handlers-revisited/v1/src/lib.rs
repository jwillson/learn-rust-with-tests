use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::Deserialize;

// ANCHOR: types
#[derive(Debug, Clone, Deserialize)]
pub struct NewUser {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub enum RegisterError {
    AlreadyExists,
    Unavailable(String),
}

/// The "ServiceThing" that does the important business logic. The handler
/// depends on this trait, not on any database.
pub trait UserService: Send + Sync {
    fn register(&self, user: NewUser) -> Result<String, RegisterError>;
}
// ANCHOR_END: types

// ANCHOR: handler
pub fn user_server(service: Arc<dyn UserService>) -> Router {
    Router::new()
        .route("/users", post(register_user))
        .with_state(service)
}

async fn register_user(
    State(service): State<Arc<dyn UserService>>,
    Json(new_user): Json<NewUser>,
) -> Response {
    match service.register(new_user) {
        Ok(id) => (StatusCode::CREATED, id).into_response(),
        Err(RegisterError::AlreadyExists) => StatusCode::CONFLICT.into_response(),
        Err(RegisterError::Unavailable(_)) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
// ANCHOR_END: handler

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::sync::Mutex;
    use tower::ServiceExt;

    // ANCHOR: mock
    struct MockUserService {
        registered: Mutex<Vec<NewUser>>,
        result: fn() -> Result<String, RegisterError>,
    }

    impl UserService for MockUserService {
        fn register(&self, user: NewUser) -> Result<String, RegisterError> {
            self.registered.lock().unwrap().push(user);
            (self.result)()
        }
    }
    // ANCHOR_END: mock

    async fn post_user(server: Router, body: &str) -> (StatusCode, String) {
        let request = Request::post("/users")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = server.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    // ANCHOR: happy_test
    #[tokio::test]
    async fn registers_a_valid_user_and_returns_201_with_the_id() {
        let service = Arc::new(MockUserService {
            registered: Mutex::new(Vec::new()),
            result: || Ok("user-42".to_string()),
        });
        let server = user_server(service.clone());

        let (status, body) = post_user(server, r#"{"name": "Ruth"}"#).await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body, "user-42");
        assert_eq!(service.registered.lock().unwrap()[0].name, "Ruth");
    }
    // ANCHOR_END: happy_test

    // ANCHOR: sad_tests
    #[tokio::test]
    async fn a_malformed_payload_is_a_400() {
        let service = Arc::new(MockUserService {
            registered: Mutex::new(Vec::new()),
            result: || Ok("unused".to_string()),
        });
        let server = user_server(service.clone());

        let (status, _) = post_user(server, "not json").await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        // The service was never even reached.
        assert!(service.registered.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_duplicate_user_is_a_409() {
        let service = Arc::new(MockUserService {
            registered: Mutex::new(Vec::new()),
            result: || Err(RegisterError::AlreadyExists),
        });
        let server = user_server(service);

        let (status, _) = post_user(server, r#"{"name": "Ruth"}"#).await;

        assert_eq!(status, StatusCode::CONFLICT);
    }
    // ANCHOR_END: sad_tests
}
