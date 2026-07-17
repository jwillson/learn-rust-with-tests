use std::fmt;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ANCHOR: error
#[derive(Debug, PartialEq)]
pub enum GetterError {
    Fetch { url: String },
    BadStatus { url: String, status: u16 },
}

impl fmt::Display for GetterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetterError::Fetch { url } => write!(f, "problem fetching from {url}"),
            GetterError::BadStatus { url, status } => {
                write!(f, "did not get 200 from {url}, got {status}")
            }
        }
    }
}

impl std::error::Error for GetterError {}
// ANCHOR_END: error

// ANCHOR: code
pub async fn dumb_getter(addr: &str) -> Result<String, GetterError> {
    let (status, body) = fetch(addr).await.map_err(|_| GetterError::Fetch {
        url: addr.to_string(),
    })?;

    if status != 200 {
        return Err(GetterError::BadStatus {
            url: addr.to_string(),
            status,
        });
    }

    Ok(body)
}

async fn fetch(addr: &str) -> std::io::Result<(u16, String)> {
    let mut stream = tokio::net::TcpStream::connect(addr).await?;
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    let status = response
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no status line"))?;

    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default();

    Ok((status, body))
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::http::StatusCode;
    use axum::routing::get;

    async fn start_server(status: StatusCode, body: &'static str) -> String {
        let app = Router::new().route("/", get(move || async move { (status, body) }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        addr.to_string()
    }

    // ANCHOR: status_test
    #[tokio::test]
    async fn a_non_200_response_is_a_typed_status_error() {
        let addr = start_server(StatusCode::IM_A_TEAPOT, "").await;

        let err = dumb_getter(&addr).await.unwrap_err();

        // We match on the *type and data*, not a string.
        assert_eq!(
            err,
            GetterError::BadStatus {
                url: addr,
                status: 418,
            }
        );
    }
    // ANCHOR_END: status_test

    // ANCHOR: other_tests
    #[tokio::test]
    async fn a_200_response_returns_the_body() {
        let addr = start_server(StatusCode::OK, "hello, world").await;

        let got = dumb_getter(&addr).await.unwrap();

        assert_eq!(got, "hello, world");
    }

    #[tokio::test]
    async fn an_unreachable_server_is_a_fetch_error() {
        // Nothing is listening on this port.
        let err = dumb_getter("127.0.0.1:1").await.unwrap_err();

        assert!(matches!(err, GetterError::Fetch { .. }), "{err:?}");
    }
    // ANCHOR_END: other_tests
}
