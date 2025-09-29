use crate::{crypto, message::*};
use anyhow::{Context, Result};
use quinn::{Connection, Endpoint, ServerConfig};
use std::{collections::HashMap, sync::Arc};
use tokio::{io::{AsyncBufReadExt, BufReader}, sync::RwLock};
use tracing::{error, info, warn};

pub struct Server {
    server_id: String,
    port: u16,
    endpoint: Endpoint,
    peers: Arc<RwLock<HashMap<String, Connection>>>,
}

impl Server {
    pub fn new(server_id: String, port: u16) -> Result<Self> {
        let cert_config = crypto::CertConfig::get_or_create()
            .context("Failed to get or create certificate")?;

        let server_config = crypto::create_server_config(cert_config)
            .context("Failed to create server config")?;

        let bind_addr = format!("0.0.0.0:{}", port);
        let endpoint = Endpoint::server(
            ServerConfig::with_crypto(Arc::new(server_config)),
            bind_addr.parse()?,
        ).context("Failed to create server endpoint")?;

        info!("服务器 {} 启动，监听地址: {}", server_id, bind_addr);

        Ok(Self {
            server_id,
            port,
            endpoint,
            peers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(&self) -> Result<()> {
        println!("🚀 服务器 '{}' 启动在端口 {}", self.server_id, self.port);
        println!("等待其他服务器连接...");
        println!("输入消息开始广播，输入 '/quit' 退出");
        println!("─────────────────────────────");

        let accept_task = {
            let endpoint = self.endpoint.clone();
            let peers = Arc::clone(&self.peers);
            let server_id = self.server_id.clone();
            tokio::spawn(async move {
                Self::handle_incoming_connections(endpoint, peers, server_id).await;
            })
        };

        let input_task = {
            let peers = Arc::clone(&self.peers);
            let server_id = self.server_id.clone();
            tokio::spawn(async move {
                Self::handle_user_input(peers, server_id).await;
            })
        };

        let _ = tokio::try_join!(accept_task, input_task);
        Ok(())
    }

    async fn handle_incoming_connections(
        endpoint: Endpoint,
        peers: Arc<RwLock<HashMap<String, Connection>>>,
        server_id: String,
    ) {
        while let Some(conn) = endpoint.accept().await {
            let connection = match conn.await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("连接失败: {}", e);
                    continue;
                }
            };

            let remote_addr = connection.remote_address();
            info!("新连接来自: {}", remote_addr);

            let peers = Arc::clone(&peers);
            let server_id = server_id.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(connection, peers, server_id).await {
                    error!("处理连接错误: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        connection: Connection,
        peers: Arc<RwLock<HashMap<String, Connection>>>,
        local_server_id: String,
    ) -> Result<()> {
        let remote_addr = connection.remote_address();
        let mut peer_id = None;

        loop {
            match connection.accept_uni().await {
                Ok(mut recv) => {
                    match Self::receive_message(&mut recv).await {
                        Ok(message) => {
                            match &message.message_type {
                                MessageType::Hello { server_id } => {
                                    info!("服务器 {} 加入", server_id);
                                    println!("📥 服务器 '{}' 加入聊天室", server_id);
                                    
                                    peer_id = Some(server_id.clone());
                                    peers.write().await.insert(server_id.clone(), connection.clone());
                                    
                                    let welcome = Message::new(
                                        local_server_id.clone(),
                                        MessageType::Welcome { server_id: local_server_id.clone() }
                                    );
                                    let _ = Self::send_message(&connection, welcome).await;
                                }
                                MessageType::Welcome { server_id } => {
                                    println!("✅ 收到服务器 '{}' 的欢迎", server_id);
                                }
                                MessageType::Text { content } => {
                                    let display = message.format_display();
                                    if !display.is_empty() {
                                        println!("{}", display);
                                    }
                                }
                                MessageType::Ping => {
                                    let pong = Message::new(local_server_id.clone(), MessageType::Pong);
                                    let _ = Self::send_message(&connection, pong).await;
                                }
                                MessageType::Pong => {
                                    // 心跳响应，不需要处理
                                }
                            }
                        }
                        Err(e) => {
                            warn!("接收消息失败 from {}: {}", remote_addr, e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("接受流失败 from {}: {}", remote_addr, e);
                    break;
                }
            }
        }

        if let Some(id) = peer_id {
            peers.write().await.remove(&id);
            println!("📤 服务器 '{}' 离开聊天室", id);
        }

        Ok(())
    }

    async fn handle_user_input(
        peers: Arc<RwLock<HashMap<String, Connection>>>,
        server_id: String,
    ) {
        let stdin = tokio::io::stdin();
        let mut lines = BufReader::new(stdin).lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            let input = line.trim();
            
            if input == "/quit" {
                info!("服务器退出");
                std::process::exit(0);
            }
            
            if input.is_empty() {
                continue;
            }
            
            let message = Message::new(
                server_id.clone(),
                MessageType::Text { content: input.to_string() }
            );
            
            let peers_read = peers.read().await;
            if peers_read.is_empty() {
                println!("⚠️  没有连接的服务器");
            } else {
                println!("📤 发送消息给 {} 个服务器", peers_read.len());
                for (peer_id, connection) in peers_read.iter() {
                    if let Err(e) = Self::send_message(connection, message.clone()).await {
                        error!("发送消息到 {} 失败: {}", peer_id, e);
                    }
                }
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
}
