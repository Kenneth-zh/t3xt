use quinn::{ClientConfig, Endpoint};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, BufReader};
use anyhow::Result;

pub async fn run(addr: String) -> Result<()> {
    let addr: SocketAddr = addr.parse()?;

    // 信任自签证书
    let cert = std::fs::read("certs/cert.der")?;
    let cert = quinn::Certificate::from_der(&cert)?;
    let mut roots = rustls::RootCertStore::empty();
    roots.add(&cert)?;
    let mut client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();
    client_crypto.alpn_protocols = vec![b"hq-29".to_vec()];

    let mut client_config = ClientConfig::new(Arc::new(client_crypto));

    let endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    let quinn::NewConnection { connection, .. } = endpoint
        .connect_with(client_config, addr, "localhost")?
        .await?;

    println!("Connected to server {}", addr);

    let (mut send, mut recv) = connection.open_bi().await?;

    // 从 stdin 读取消息并发送
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        send.write_all(line.as_bytes()).await?;
        send.write_all(b"\n").await?;

        let mut buf = vec![0; 1024];
        if let Ok(n) = recv.read(&mut buf).await {
            if n > 0 {
                println!("Server reply: {}", String::from_utf8_lossy(&buf[..n]));
            }
        }
    }

    Ok(())
}
