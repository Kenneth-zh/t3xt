use crate::{crypto, message::*};
use anyhow::{Context, Result};
use quinn::{Connection, Endpoint, ServerConfig};
use std::sync::Arc;
use tokio::{io::{AsyncBufReadExt, BufReader}, sync::RwLock};
use tracing::{error, info, warn};

pub struct Server {
    server_id: String,
    port: u16,
    endpoint: Endpoint,
    peers: Arc<RwLock<Vec<Connection>>>,
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
            peers: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn run(&self) -> Result<()> {
        println!("服务器 '{}' 启动在端口 {}", self.server_id, self.port);
        println!("等待客户端连接...");
        println!("输入消息开始广播，输入 '/quit' 退出");
        println!("─────────────────────────────");

        let accept_task = {
            let endpoint = self.endpoint.clone();
            let peers = Arc::clone(&self.peers);
            tokio::spawn(async move {
                Self::handle_incoming_connections(endpoint, peers).await;
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
        peers: Arc<RwLock<Vec<Connection>>>,
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
            
            // 将连接加入 peers
            {
                let mut peers_guard = peers.write().await;
                peers_guard.push(connection.clone());
            }
            println!("新客户端连接: {}", remote_addr);

            // 启动处理该连接的任务
            let peers = Arc::clone(&peers);
            let peer_addr = remote_addr.to_string();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(connection, peers, peer_addr).await {
                    error!("处理连接错误: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        connection: Connection,
        peers: Arc<RwLock<Vec<Connection>>>,
        peer_addr: String,
    ) -> Result<()> {
        loop {
            match connection.accept_uni().await {
                Ok(mut recv) => {
                    match Self::receive_message(&mut recv).await {
                        Ok(message) => {
                            // 只处理文本消息
                            match &message.message_type {
                                MessageType::Text { content } => {
                                    println!("[{}]: {}", message.sender_id, content);

                                    // 广播给其他连接的客户端（不包括发送者）
                                    let peers_read = peers.read().await;
                                    for other_conn in peers_read.iter() {
                                        if other_conn.remote_address().to_string() != peer_addr {
                                            let _ = Self::send_message(other_conn, message.clone()).await;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("解析消息失败 from {}: {}", peer_addr, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("接受流失败 from {}: {}", peer_addr, e);
                    break;
                }
            }
        }
        
        {
            let mut peers_guard = peers.write().await;
            peers_guard.retain(|conn| conn.remote_address().to_string() != peer_addr);
        }
        println!("客户端 '{}' 断开连接", peer_addr);

        Ok(())
    }

    async fn handle_user_input(
        peers: Arc<RwLock<Vec<Connection>>>,
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
            
            let message = Message::new_text(server_id.clone(), input.to_string());
            
            let peers_read = peers.read().await;
            if peers_read.is_empty() {
                println!("没有连接的客户端");
            } else {
                println!("发送消息给 {} 个客户端", peers_read.len());
                for connection in peers_read.iter() {
                    if let Err(e) = Self::send_message(connection, message.clone()).await {
                        warn!("发送消息失败: {}", e);
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