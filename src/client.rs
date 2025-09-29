use crate::{crypto, message::*};
use anyhow::{Context, Result};
use quinn::{ClientConfig, Connection, Endpoint};
use std::sync::Arc;
use tokio::{io::{AsyncBufReadExt, BufReader}, time::{sleep, Duration}};
use tracing::{error, info, warn};

pub struct Client {
    server_id: String,
    endpoint: Endpoint,
    connection: Option<Connection>,
}

impl Client {
    pub fn new(client_id: String) -> Result<Self> {
        // 尝试使用服务器证书，如果不存在则使用不安全模式
        let client_config = if std::path::Path::new("certs/server.crt").exists() {
            println!("🔐 使用服务器证书进行安全连接");
            let rustls_config = crypto::create_client_config_with_cert("certs/server.crt")?;
            crypto::create_quinn_client_config(rustls_config)
        } else {
            println!("⚠️  使用不安全模式连接（跳过证书验证）");
            let rustls_config = crypto::create_insecure_client_config()?;
            crypto::create_quinn_client_config(rustls_config)
        };

        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        Ok(Self {
            server_id: client_id,
            endpoint,
            connection: None,
        })
    }

    pub async fn connect(&mut self, target_addr: &str, target_port: u16) -> Result<()> {
        let addr = format!("{}:{}", target_addr, target_port);
        
        println!("正在连接到服务器 {}...", addr);
        info!("Connecting to server at {}", addr);

        let connection = self
            .endpoint
            .connect(addr.parse()?, "localhost")?
            .await
            .context("Failed to connect to server")?;

        println!("✅ 成功连接到服务器！");

        // 发送Hello消息
        let hello_msg = Message::new(
            self.server_id.clone(),
            MessageType::Hello { server_id: self.server_id.clone() },
        );

        Self::send_message(&connection, hello_msg).await?;
        self.connection = Some(connection);

        Ok(())
    }

    pub async fn run_interactive(&self) -> Result<()> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("未连接到服务器"))?;

        println!("📝 连接成功！可以开始发送消息，输入 '/quit' 退出");
        println!("─────────────────────────────");
        
        // 启动接收消息的任务
        let recv_task = {
            let connection = connection.clone();
            tokio::spawn(async move {
                Self::handle_incoming_messages(connection).await;
            })
        };
        
        // 启动用户输入任务
        let input_task = {
            let connection = connection.clone();
            let server_id = self.server_id.clone();
            tokio::spawn(async move {
                Self::handle_user_input(connection, server_id).await;
            })
        };

        // 启动心跳任务
        let ping_task = {
            let connection = connection.clone();
            let server_id = self.server_id.clone();
            tokio::spawn(async move {
                Self::send_ping_periodically(connection, server_id).await;
            })
        };

        let _ = tokio::try_join!(recv_task, input_task, ping_task);
        Ok(())
    }

    async fn handle_incoming_messages(connection: Connection) {
        loop {
            match connection.accept_uni().await {
                Ok(mut recv) => {
                    match Self::receive_message(&mut recv).await {
                        Ok(message) => {
                            match &message.message_type {
                                MessageType::Welcome { server_id } => {
                                    println!("🎉 收到服务器 '{}' 的欢迎消息", server_id);
                                }
                                MessageType::Text { content: _ } => {
                                    let display = message.format_display();
                                    if !display.is_empty() {
                                        println!("{}", display);
                                    }
                                }
                                MessageType::Ping => {
                                    // 接收到ping，发送pong
                                    let pong = Message::new("client".to_string(), MessageType::Pong);
                                    let _ = Self::send_message(&connection, pong).await;
                                }
                                MessageType::Pong => {
                                    // 收到心跳响应
                                    info!("收到心跳响应");
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            warn!("接收消息失败: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("接受流失败: {}", e);
                    break;
                }
            }
        }
    }

    async fn handle_user_input(connection: Connection, server_id: String) {
        let stdin = tokio::io::stdin();
        let mut lines = BufReader::new(stdin).lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            let input = line.trim();
            
            if input == "/quit" {
                info!("客户端退出");
                std::process::exit(0);
            }
            
            if input.is_empty() {
                continue;
            }
            
            let message = Message::new(
                server_id.clone(),
                MessageType::Text { content: input.to_string() }
            );
            
            if let Err(e) = Self::send_message(&connection, message).await {
                error!("发送消息失败: {}", e);
            } else {
                println!("✅ 消息已发送");
            }
        }
    }

    async fn send_ping_periodically(connection: Connection, server_id: String) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            let ping = Message::new(server_id.clone(), MessageType::Ping);
            if let Err(e) = Self::send_message(&connection, ping).await {
                error!("发送心跳失败: {}", e);
                break;
            }
        }
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

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(connection) = self.connection.take() {
            connection.close(0u32.into(), b"Goodbye");
        }
        Ok(())
    }
}
