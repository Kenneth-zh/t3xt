use crate::{crypto, message::*};
use anyhow::{Context, Result};
use quinn::{Connection, Endpoint};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc,
};
use tracing::{info, warn};

pub struct Client {
    client_id: String,
    endpoint: Endpoint,
    connection: Option<Connection>,
}

impl Client {
    pub fn new(client_id: String) -> Result<Self> {
        let cert_path = std::path::Path::new("certs/server.crt");
        if !cert_path.exists() {
            return Err(anyhow::anyhow!(
                "certs/server.crt not found."
            ));
        }

        println!("found cert");   
        let rustls_config = crypto::create_client_config_with_cert(cert_path)?;
        let client_config = crypto::create_quinn_client_config(rustls_config);

        // 如果你有多网卡或需要指定出口IP，可以将 "0.0.0.0" 替换为具体的本地IP。
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        Ok(Self {
            client_id,
            endpoint,
            connection: None,
        })
    }

    pub async fn connect(&mut self, server_addr: &str, port: u16) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", server_addr, port).parse()
            .context("Invalid server address")?;
        
        info!("connect to {}", addr);
        println!("connecting to {}...", addr);

        let connection = self.endpoint
            .connect(addr, "localhost")?
            .await
            .context("Failed to establish connection")?;
        println!("connected");
        
        self.connection = Some(connection);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(connection) = &self.connection {
            connection.close(0u32.into(), b"Goodbye");
            self.connection = None;
            println!("disconnected");
        }
        Ok(())
    }

    pub async fn run_interactive(&mut self) -> Result<()> {

        // 为信息队列准备
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        
        let recv_connection = self.connection.as_ref().unwrap().clone();
        let recv_task = tokio::spawn(async move {
            loop{
                match recv_connection.accept_uni().await{
                    Ok(mut recvstream) => {
                        match Self::receive_message(&mut recvstream).await{
                            Ok(message) => {
                                println!("{}", message.format_display());
                            }
                            Err(e) => {
                                warn!("Failed to receive message: {}", e);
                                break;
                        }
                    }
                    }  
                    Err(_) => {
                        break;
                    }
                    }
                }
            
        });
        
        let send_connection = self.connection.as_ref().unwrap().clone();
        let send_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if Self::send_message(&send_connection, message).await.is_err() {
                    break;
                }
            }
        });
        
        // 用户输入处理
        println!("输入消息并按回车发送，输入 '/quit' 退出");
        println!("─────────────────────────────────────");
        
        let stdin = tokio::io::stdin();
        let mut lines = BufReader::new(stdin).lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            let input = line.trim();
            
            if input == "/quit" {
                break;
            }
            
            if input.is_empty() {
                continue;
            }
            
            let message = Message::new_text(self.client_id.clone(), input.to_string());
            
            if tx.send(message).is_err() {
                break;
            }
        }
        
        // 清理任务
        recv_task.abort();
        send_task.abort();
        
        Ok(())
    }

    async fn send_message(connection: &Connection, message: Message) -> Result<()> {
        let mut send = connection.open_uni().await
            .context("Failed to open stream")?;
        
        let data = message.to_bytes()?;
        send.write_all(&data).await
            .context("Failed to send message")?;
        
        send.finish().await
            .context("Failed to finish stream")?;
        
        Ok(())
    }

    async fn receive_message(recv: &mut quinn::RecvStream) -> Result<Message> {
        let data = recv.read_to_end(8192).await
            .context("Failed to read message")?;
        
        Message::from_bytes(&data)
    }
}