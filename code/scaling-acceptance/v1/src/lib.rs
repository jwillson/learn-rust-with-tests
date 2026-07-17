use axum::Router;
use axum::routing::get;

// ANCHOR: specification
#[allow(async_fn_in_trait)] // static dispatch only; we never need Send bounds on the future
pub trait Greeter {
    async fn greet(&self) -> String;
}

pub async fn greet_specification(greeter: &impl Greeter) {
    let got = greeter.greet().await;
    assert_eq!(got, "Hello, world");
}
// ANCHOR_END: specification

// ANCHOR: domain
pub fn greet() -> String {
    "Hello, world".to_string()
}

pub struct DomainGreeter;

impl Greeter for DomainGreeter {
    async fn greet(&self) -> String {
        greet()
    }
}
// ANCHOR_END: domain

// ANCHOR: server
pub fn greet_server() -> Router {
    Router::new().route("/greet", get(greet_handler))
}

async fn greet_handler() -> String {
    greet()
}
// ANCHOR_END: server

// ANCHOR: driver
pub struct HttpGreeterDriver {
    pub base_url: String,
}

impl Greeter for HttpGreeterDriver {
    async fn greet(&self) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let host = self.base_url.trim_start_matches("http://");
        let mut stream = tokio::net::TcpStream::connect(host)
            .await
            .expect("failed to connect to the greet server");
        stream
            .write_all(b"GET /greet HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .expect("failed to send request");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .expect("failed to read response");

        // The body is everything after the blank line separating headers.
        response
            .split_once("\r\n\r\n")
            .map(|(_, body)| body.to_string())
            .unwrap_or_default()
    }
}
// ANCHOR_END: driver

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: domain_test
    #[tokio::test]
    async fn greeter_domain_satisfies_the_specification() {
        greet_specification(&DomainGreeter).await;
    }
    // ANCHOR_END: domain_test

    // ANCHOR: http_test
    #[tokio::test]
    async fn greeter_over_http_satisfies_the_specification() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, greet_server()).await.unwrap();
        });

        let driver = HttpGreeterDriver {
            base_url: format!("http://{addr}"),
        };

        greet_specification(&driver).await;
    }
    // ANCHOR_END: http_test
}
