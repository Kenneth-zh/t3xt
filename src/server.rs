use crate::{crypto, message};
use anyhow::{Context, Result};
use quinn::{Endpoint, ServerConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// QUIC服务器
pub struct Server {
    pub endpoint: Endpoint,
    pub session: Arc<RwLock<message::Session>>,
}

impl Server {
    /// 创建新的服务器实例
    pub fn new(bind_addr: &str, port: u16) -> Result<Self> {
        // 生成自签名证书
        let cert_config = crypto::CertConfig::generate_self_signed()
            .context("Failed to generate certificate")?;

        // 创建TLS配置
        let tls_config = crypto::create_server_config(cert_config)
            .context("Failed to create TLS config")?;

        // 创建QUIC服务器配置
        let server_config = ServerConfig::with_crypto(Arc::new(tls_config));

        // 绑定地址
        let bind_addr = format!("{}:{}", bind_addr, port);
        
        // 创建endpoint
        let endpoint = Endpoint::server(server_config, bind_addr.parse()?)
            .context("Failed to create endpoint")?;

        info!("Server listening on {}", bind_addr);

        Ok(Self {
            endpoint,
            session: Arc::new(RwLock::new(message::Session::new())),
        })
    }

    /// 启动服务器并处理连接
    pub async fn run(&self) -> Result<()> {
        info!("Server started, waiting for connections...");

        while let Some(conn) = self.endpoint.accept().await {
            let session = Arc::clone(&self.session);
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(conn, session).await {
                    error!("Connection failed: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 处理单个连接
    async fn handle_connection(
        conn: quinn::Connecting,
        session: Arc<RwLock<message::Session>>,
    ) -> Result<()> {
        let connection = conn.await.context("Failed to establish connection")?;
        let remote_addr = connection.remote_address();
        info!("New connection from {}", remote_addr);

        // 处理双向流
        loop {
            tokio::select! {
                stream = connection.accept_bi() => {
                    match stream {
                        Ok((mut send, mut recv)) => {
                            let session = Arc::clone(&session);
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_stream(&mut send, &mut recv, session).await {
                                    warn!("Stream error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept stream: {}", e);
                            break;
                        }
                    }
                }
                _ = connection.closed() => {
                    info!("Connection {} closed", remote_addr);
                    break;
                }
            }
        }

        Ok(())
    }

    /// 处理数据流
    async fn handle_stream(
        send: &mut quinn::SendStream,
        recv: &mut quinn::RecvStream,
        session: Arc<RwLock<message::Session>>,
    ) -> Result<()> {
        // 读取消息
        let buffer = recv.read_to_end(10 * 1024 * 1024).await
            .context("Failed to read stream")?;

        if buffer.is_empty() {
            return Ok(());
        }

        // 解析消息
        let message = message::Message::from_bytes(&buffer)
            .context("Failed to parse message")?;

        info!("Received message from {}: {:?}", message.sender, message.message_type);

        // 处理消息
        let response = Self::process_message(message, &session).await?;

        // 发送响应
        if let Some(response_msg) = response {
            let response_bytes = response_msg.to_bytes()
                .context("Failed to serialize response")?;
            send.write_all(&response_bytes).await
                .context("Failed to send response")?;
            send.finish().await
                .context("Failed to finish sending")?;
        }

        Ok(())
    }

    /// 处理接收到的消息
    async fn process_message(
        message: message::Message,
        session: &Arc<RwLock<message::Session>>,
    ) -> Result<Option<message::Message>> {
        match &message.message_type {
            message::MessageType::Text(text) => {
                info!("Text message from {}: {}", message.sender, text);
                
                // 广播消息给所有连接的用户
                Self::broadcast_message(&message, session).await?;
                
                // 返回确认消息
                let mut session_lock = session.write().await;
                let response_id = session_lock.next_message_id();
                Ok(Some(message::Message::new(
                    response_id,
                    "server".to_string(),
                    message::MessageType::Text("Message received".to_string()),
                )))
            }
            message::MessageType::UserJoined(username) => {
                info!("User {} joined", username);
                Ok(None)
            }
            message::MessageType::UserLeft(username) => {
                info!("User {} left", username);
                let mut session_lock = session.write().await;
                session_lock.remove_user(username);
                Ok(None)
            }
            message::MessageType::Ping => {
                // 响应心跳
                let mut session_lock = session.write().await;
                let response_id = session_lock.next_message_id();
                Ok(Some(message::Message::new(
                    response_id,
                    "server".to_string(),
                    message::MessageType::Pong,
                )))
            }
            _ => {
                warn!("Unhandled message type: {:?}", message.message_type);
                Ok(None)
            }
        }
    }

    /// 广播消息给所有连接的用户
    async fn broadcast_message(
        message: &message::Message,
        session: &Arc<RwLock<message::Session>>,
    ) -> Result<()> {
        let session_lock = session.read().await;
        let message_bytes = message.to_bytes()?;

        for (username, connection) in &session_lock.users {
            if username != &message.sender {
                match connection.open_uni().await {
                    Ok(mut stream) => {
                        if let Err(e) = stream.write_all(&message_bytes).await {
                            warn!("Failed to send message to {}: {}", username, e);
                        } else if let Err(e) = stream.finish().await {
                            warn!("Failed to finish sending to {}: {}", username, e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to open stream to {}: {}", username, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取服务器统计信息
    pub async fn get_stats(&self) -> Result<String> {
        let session = self.session.read().await;
        Ok(format!(
            "Connected users: {}\nTotal messages: {}",
            session.get_user_count(),
            session.message_counter
        ))
    }
}