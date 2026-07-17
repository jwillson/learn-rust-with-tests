use axum::Router;
use axum::routing::get;

// ANCHOR: code
pub fn player_server() -> Router {
    Router::new().route("/players/{name}", get(get_score))
}

async fn get_score() -> String {
    "20".to_string()
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    // ANCHOR: helpers
    async fn get_score_request(server: Router, name: &str) -> (StatusCode, String) {
        let request = Request::get(format!("/players/{name}"))
            .body(Body::empty())
            .unwrap();

        let response = server.oneshot(request).await.unwrap();

        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(bytes.to_vec()).unwrap();

        (status, body)
    }
    // ANCHOR_END: helpers

    // ANCHOR: test
    #[tokio::test]
    async fn returns_peppers_score() {
        let (_status, body) = get_score_request(player_server(), "Pepper").await;

        assert_eq!(body, "20");
    }
    // ANCHOR_END: test
}
