//! `bardo-compute` - compute provisioning service and fleet manager.
//!
//! **Implemented by:** Plan 01
//!
//! This binary is a shell. Later plans implement the compute service.

use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("bardo-compute starting - not yet implemented");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tracing::info!(addr = %addr, "health endpoint listening");

    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let body = r#"{"status":"ok","service":"bardo-compute"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body,
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}
