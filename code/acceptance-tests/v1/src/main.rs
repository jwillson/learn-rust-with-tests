// ANCHOR: main
use acceptance_tests_v1::serve;

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    println!("shutdown signal received; draining in-flight requests");
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to port 8080");

    println!("listening on http://{}", listener.local_addr().unwrap());
    serve(listener, shutdown_signal()).await;
    println!("shutdown gracefully; all responses were sent");
}
// ANCHOR_END: main
