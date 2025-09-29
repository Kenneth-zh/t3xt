use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 简化的消息类型，专注于服务器间文本通信
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 文本消息
    Text { content: String },
    /// 连接握手
    Hello { server_id: String },
    /// 连接确认
    Welcome { server_id: String },
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
}

/// 服务器间消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub from_server: String,
    pub to_server: Option<String>, // None表示广播
    pub message_type: MessageType,
}

impl Message {
    /// 创建新消息
    pub fn new(from_server: String, message_type: MessageType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            from_server,
            to_server: None,
            message_type,
        }
    }

    /// 创建发送给特定服务器的消息
    pub fn new_to(from_server: String, to_server: String, message_type: MessageType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            from_server,
            to_server: Some(to_server),
            message_type,
        }
    }

    /// 序列化为字节数组
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let json = serde_json::to_string(self)?;
        Ok(json.into_bytes())
    }

    /// 从字节数组反序列化
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let json = String::from_utf8(bytes.to_vec())?;
        let message = serde_json::from_str(&json)?;
        Ok(message)
    }

    /// 格式化显示消息
    pub fn format_display(&self) -> String {
        match &self.message_type {
            MessageType::Text { content } => {
                format!("[{}] {}: {}", 
                    self.timestamp.format("%H:%M:%S"),
                    self.from_server,
                    content
                )
            }
            MessageType::Hello { server_id } => {
                format!("[{}] {} 加入聊天室", 
                    self.timestamp.format("%H:%M:%S"),
                    server_id
                )
            }
            MessageType::Welcome { server_id } => {
                format!("[{}] 欢迎 {} 加入", 
                    self.timestamp.format("%H:%M:%S"),
                    server_id
                )
            }
            _ => String::new(), // 心跳消息不显示
        }
    }
}