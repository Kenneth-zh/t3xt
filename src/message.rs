use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 极简消息类型 - 只保留核心功能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 文本消息 - 唯一的消息类型
    Text { content: String },
}

/// 服务器间消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub timestamp: DateTime<Utc>,
    pub sender_id: String,
    pub message_type: MessageType,
}

impl Message {
    /// 创建新文本消息
    pub fn new_text(sender_id: String, content: String) -> Self {
        Self {
            timestamp: Utc::now(),
            sender_id,
            message_type: MessageType::Text { content },
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
                    self.sender_id,
                    content
                )
            }
        }
    }
}