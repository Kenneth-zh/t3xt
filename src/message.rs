use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 消息类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 文本消息
    Text(String),
    /// 用户加入
    UserJoined(String),
    /// 用户离开
    UserLeft(String),
    /// 文件传输请求
    FileRequest { filename: String, size: u64 },
    /// 文件传输数据
    FileData { chunk_id: u32, data: Vec<u8> },
    /// 心跳包
    Ping,
    /// 心跳响应
    Pong,
}

/// 即时消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub timestamp: u64,
    pub sender: String,
    pub message_type: MessageType,
}

impl Message {
    /// 创建新消息
    pub fn new(id: u64, sender: String, message_type: MessageType) -> Self {
        Self {
            id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sender,
            message_type,
        }
    }

    /// 序列化为JSON字节数组
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
}

/// 用户会话管理
#[derive(Debug)]
pub struct Session {
    pub users: HashMap<String, quinn::Connection>,
    pub message_counter: u64,
}

impl Session {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            message_counter: 0,
        }
    }

    pub fn add_user(&mut self, username: String, connection: quinn::Connection) {
        self.users.insert(username, connection);
    }

    pub fn remove_user(&mut self, username: &str) -> Option<quinn::Connection> {
        self.users.remove(username)
    }

    pub fn next_message_id(&mut self) -> u64 {
        self.message_counter += 1;
        self.message_counter
    }

    pub fn get_user_count(&self) -> usize {
        self.users.len()
    }
}