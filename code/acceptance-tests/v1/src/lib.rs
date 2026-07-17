use std::future::Future;
use std::time::Duration;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;

// ANCHOR: code
pub async fn slow_handler() -> &'static str {
    tokio::time::sleep(Duration::from_millis(100)).await;
    "hello, world"
}

pub fn app() -> Router {
    Router::new().route("/", get(slow_handler))
}

pub async fn serve(listener: TcpListener, shutdown: impl Future<Output = ()> + Send + 'static) {
    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown)
        .await
        .expect("server error");
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::oneshot;

    // ANCHOR: helpers
    async fn get_root(addr: std::net::SocketAddr) -> std::io::Result<String> {
        let mut stream = tokio::net::TcpStream::connect(addr).await?;
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await?;

        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        Ok(response)
    }
    // ANCHOR_END: helpers

    // ANCHOR: test
    #[tokio::test]
    async fn in_flight_requests_complete_after_shutdown_is_triggered() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let server = tokio::spawn(serve(listener, async {
            shutdown_rx.await.ok();
        }));

        // Start a request; the slow handler will still be running after this returns.
        let request = tokio::spawn(get_root(addr));
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Now signal shutdown while the request is mid-flight.
        shutdown_tx.send(()).unwrap();

        // The in-flight request must still receive its full response.
        let response = request.await.unwrap().unwrap();
        assert!(response.contains("200 OK"), "{response}");
        assert!(response.contains("hello, world"), "{response}");

        // And the server task returns cleanly once it has drained.
        server.await.unwrap();

        // After shutdown, new connections are refused.
        assert!(get_root(addr).await.is_err());
    }
    // ANCHOR_END: test
}
