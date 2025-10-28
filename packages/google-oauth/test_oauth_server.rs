use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test OAuth server on 127.0.0.1:8080");
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Successfully bound to 127.0.0.1:8080");
    println!("Waiting for connection...");
    
    let (mut stream, addr) = listener.accept().await?;
    println!("Accepted connection from: {}", addr);
    
    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);
    println!("Received request:\n{}", request);
    
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>OAuth callback received!</h1></body></html>";
    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await?;
    
    println!("Response sent, server shutting down");
    Ok(())
}