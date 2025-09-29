use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 简化的消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text { content: String },
    Ping,
    Pong,
}

/// 服务器间消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub timestamp: DateTime<Utc>,
    pub sender_id: String,  // 重命名为更通用的名称
    pub message_type: MessageType,
}

impl Message {
    /// 创建新消息
    pub fn new(sender_id: String, message_type: MessageType) -> Self {
        Self {
            timestamp: Utc::now(),
            sender_id,
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

    /// 权宜之计，其实可以用Option<String>
    pub fn format_display(&self) -> String {
        match &self.message_type {
            MessageType::Text { content } => {
                format!("[{}] {}: {}", 
                    self.timestamp.format("%H:%M:%S"),
                    self.sender_id,
                    content
                )
            }
            MessageType::Ping | MessageType::Pong => {
                // 心跳消息通常不显示给用户
                String::new()
            }
        }
    }

    /// 判断是否为文本消息（需要显示给用户）
    pub fn is_text_message(&self) -> bool {
        matches!(self.message_type, MessageType::Text { .. })
    }
}