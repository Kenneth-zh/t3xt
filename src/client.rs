use crate::{crypto, message};
use anyhow::{Context, Result};
use quinn::{ClientConfig, Connection, Endpoint};
use std::{io::{self, Write}, sync::Arc, time::Duration};
use tokio::time;
use tracing::{error, info, warn};

/// QUIC客户端
pub struct Client {
    endpoint: Endpoint,
    connection: Option<Connection>,
    username: String,
    message_counter: u64,
}

impl Client {
    /// 创建新的客户端实例
    pub fn new(username: String) -> Result<Self> {
        // 创建客户端TLS配置
        let client_config = crypto::create_client_config()
            .context("Failed to create client config")?;

        // 创建QUIC客户端配置
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
            .context("Failed to create client endpoint")?;
        endpoint.set_default_client_config(ClientConfig::new(Arc::new(client_config)));

        Ok(Self {
            endpoint,
            connection: None,
            username,
            message_counter: 0,
        })
    }

    /// 连接到服务器
    pub async fn connect(&mut self, server_addr: &str, port: u16) -> Result<()> {
        let server_addr = format!("{}:{}", server_addr, port);
        info!("Connecting to server at {}", server_addr);

        let connection = self
            .endpoint
            .connect(server_addr.parse()?, "localhost")?
            .await
            .context("Failed to connect to server")?;

        info!("Connected to server");

        // 发送用户加入消息
        let join_message = message::Message::new(
            self.next_message_id(),
            self.username.clone(),
            message::MessageType::UserJoined(self.username.clone()),
        );

        self.send_message(&connection, join_message).await?;
        self.connection = Some(connection);

        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<()> {
        let connection = self.connection.take();
        
        if let Some(connection) = connection {
            // 发送用户离开消息
            let message_id = self.next_message_id();
            let leave_message = message::Message::new(
                message_id,
                self.username.clone(),
                message::MessageType::UserLeft(self.username.clone()),
            );

            if let Err(e) = self.send_message(&connection, leave_message).await {
                warn!("Failed to send leave message: {}", e);
            }

            connection.close(0u32.into(), b"Client disconnecting");
        }

        info!("Disconnected from server");
        Ok(())
    }

    /// 发送文本消息
    pub async fn send_text(&mut self, text: String) -> Result<()> {
        let connection = match &self.connection {
            Some(conn) => conn.clone(),
            None => return Err(anyhow::anyhow!("Not connected to server")),
        };
        
        let message_id = self.next_message_id();
        let message = message::Message::new(
            message_id,
            self.username.clone(),
            message::MessageType::Text(text),
        );

        self.send_message(&connection, message).await?;
        Ok(())
    }

    /// 发送消息到服务器
    async fn send_message(&self, connection: &Connection, message: message::Message) -> Result<()> {
        let (mut send, mut recv) = connection.open_bi().await
            .context("Failed to open stream")?;

        let message_bytes = message.to_bytes()
            .context("Failed to serialize message")?;

        send.write_all(&message_bytes).await
            .context("Failed to send message")?;
        send.finish().await
            .context("Failed to finish sending")?;

        // 读取服务器响应
        let response_buffer = recv.read_to_end(1024 * 1024).await
            .context("Failed to read response")?;

        if !response_buffer.is_empty() {
            if let Ok(response) = message::Message::from_bytes(&response_buffer) {
                info!("Server response: {:?}", response.message_type);
            }
        }

        Ok(())
    }

    /// 监听来自服务器的消息
    pub async fn listen_for_messages(&self) -> Result<()> {
        if let Some(connection) = &self.connection {
            loop {
                tokio::select! {
                    stream = connection.accept_uni() => {
                        match stream {
                            Ok(mut recv) => {
                                if let Ok(buffer) = recv.read_to_end(1024 * 1024).await {
                                    if let Ok(message) = message::Message::from_bytes(&buffer) {
                                        self.handle_received_message(message);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to accept uni stream: {}", e);
                                break;
                            }
                        }
                    }
                    _ = connection.closed() => {
                        info!("Server connection closed");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// 处理接收到的消息
    fn handle_received_message(&self, message: message::Message) {
        match message.message_type {
            message::MessageType::Text(text) => {
                println!("[{}] {}: {}", 
                    Self::format_timestamp(message.timestamp),
                    message.sender, 
                    text
                );
            }
            message::MessageType::UserJoined(username) => {
                println!("* {} joined the chat", username);
            }
            message::MessageType::UserLeft(username) => {
                println!("* {} left the chat", username);
            }
            message::MessageType::Pong => {
                info!("Received pong from server");
            }
            _ => {
                info!("Received message: {:?}", message.message_type);
            }
        }
    }

    /// 启动心跳机制
    pub async fn start_heartbeat(&self) -> Result<()> {
        if let Some(connection) = &self.connection {
            let connection = connection.clone();
            let username = self.username.clone();
            
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(30));
                let mut counter = 0u64;

                loop {
                    interval.tick().await;

                    counter += 1;
                    let ping_message = message::Message::new(
                        counter,
                        username.clone(),
                        message::MessageType::Ping,
                    );

                    match Self::send_heartbeat(&connection, ping_message).await {
                        Ok(_) => {
                            info!("Sent heartbeat");
                        }
                        Err(e) => {
                            warn!("Failed to send heartbeat: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }

    /// 发送心跳消息
    async fn send_heartbeat(connection: &Connection, message: message::Message) -> Result<()> {
        let (mut send, _recv) = connection.open_bi().await
            .context("Failed to open heartbeat stream")?;

        let message_bytes = message.to_bytes()
            .context("Failed to serialize heartbeat")?;

        send.write_all(&message_bytes).await
            .context("Failed to send heartbeat")?;
        send.finish().await
            .context("Failed to finish heartbeat")?;

        Ok(())
    }

    /// 启动交互式聊天
    pub async fn start_interactive_chat(&mut self) -> Result<()> {
        println!("Connected as {}. Type messages and press Enter to send. Type 'quit' to exit.", self.username);
        
        // 启动消息监听任务
        let connection = self.connection.as_ref().unwrap().clone();
        let listen_task = {
            let client_clone = Self {
                endpoint: self.endpoint.clone(),
                connection: Some(connection.clone()),
                username: self.username.clone(),
                message_counter: 0,
            };
            tokio::spawn(async move {
                if let Err(e) = client_clone.listen_for_messages().await {
                    error!("Message listening failed: {}", e);
                }
            })
        };

        // 启动心跳任务
        let heartbeat_task = {
            let client_clone = Self {
                endpoint: self.endpoint.clone(),
                connection: Some(connection),
                username: self.username.clone(),
                message_counter: 0,
            };
            tokio::spawn(async move {
                if let Err(e) = client_clone.start_heartbeat().await {
                    error!("Heartbeat failed: {}", e);
                }
            })
        };

        // 处理用户输入
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim().to_string();
                    if input.is_empty() {
                        continue;
                    }
                    
                    if input == "quit" {
                        break;
                    }

                    if let Err(e) = self.send_text(input).await {
                        error!("Failed to send message: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to read input: {}", e);
                    break;
                }
            }
        }

        // 取消后台任务
        listen_task.abort();
        heartbeat_task.abort();

        Ok(())
    }

    /// 格式化时间戳
    fn format_timestamp(timestamp: u64) -> String {
        let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        datetime.format("%H:%M:%S").to_string()
    }

    /// 获取下一个消息ID
    fn next_message_id(&mut self) -> u64 {
        self.message_counter += 1;
        self.message_counter
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}