# T3XT - QUIC点对点即时通信软件

T3XT是一个基于QUIC协议的高性能点对点即时通信软件，使用Rust编写，提供安全、快速的实时消息传输。

## 特性

- 🚀 **高性能**: 基于QUIC协议，支持多路复用和0-RTT连接
- 🔒 **安全加密**: 内置TLS 1.3加密，确保通信安全
- 📱 **实时通信**: 支持实时文本消息传输
- 💪 **低延迟**: QUIC协议的天然优势，减少握手延迟
- 🔧 **易于使用**: 简单的命令行界面
- 📊 **心跳机制**: 自动检测连接状态
- 🌐 **点对点**: 支持多客户端同时连接

## 安装

确保您已安装Rust（1.70+）：

```bash
# 克隆项目
git clone <your-repo>
cd t3xt

# 编译项目
cargo build --release
```

## 使用方法

### 启动服务器

```bash
# 使用默认设置（127.0.0.1:8080）
cargo run -- server

# 指定绑定地址和端口
cargo run -- server --bind 0.0.0.0 --port 9090
```

### 连接客户端

```bash
# 连接到本地服务器
cargo run -- client --username Alice

# 连接到远程服务器
cargo run -- client --username Bob --server 192.168.1.100 --port 9090
```

## 命令行参数

### 服务器模式

```
cargo run -- server [选项]

选项:
  -b, --bind <BIND>    绑定地址 [默认: 127.0.0.1]
  -p, --port <PORT>    监听端口 [默认: 8080]
  -h, --help           显示帮助信息
```

### 客户端模式

```
cargo run -- client [选项]

选项:
  -u, --username <USERNAME>  用户名 [必需]
  -s, --server <SERVER>      服务器地址 [默认: 127.0.0.1]
  -p, --port <PORT>          服务器端口 [默认: 8080]
  -h, --help                 显示帮助信息
```

## 使用示例

### 1. 本地测试

**终端1 - 启动服务器：**
```bash
cargo run -- server
```

**终端2 - 客户端1：**
```bash
cargo run -- client --username Alice
```

**终端3 - 客户端2：**
```bash
cargo run -- client --username Bob
```

### 2. 网络部署

**服务器（公网IP: 192.168.1.100）：**
```bash
cargo run -- server --bind 0.0.0.0 --port 8080
```

**客户端：**
```bash
cargo run -- client --username Alice --server 192.168.1.100 --port 8080
```

## 聊天操作

连接成功后，您可以：

- 输入文本消息并按Enter发送
- 输入 `quit` 退出聊天
- 消息会实时广播给所有在线用户

## 技术架构

### 核心组件

1. **QUIC传输**: 使用 `quinn` 库实现QUIC协议
2. **TLS加密**: 基于 `rustls` 提供TLS 1.3加密
3. **消息序列化**: 使用 `serde_json` 进行消息序列化
4. **异步运行时**: 基于 `tokio` 异步运行时

### 消息类型

- `Text(String)`: 文本消息
- `UserJoined(String)`: 用户加入通知
- `UserLeft(String)`: 用户离开通知
- `FileRequest`: 文件传输请求（预留）
- `FileData`: 文件传输数据（预留）
- `Ping/Pong`: 心跳检测

### 安全特性

- **自动证书生成**: 服务器自动生成自签名证书
- **TLS 1.3加密**: 所有通信都经过TLS 1.3加密
- **连接验证**: 客户端连接需要通过握手验证
- **消息完整性**: 确保消息在传输过程中不被篡改

## 性能优化

- **0-RTT连接**: 支持快速重连
- **多路复用**: 单连接支持多个数据流
- **拥塞控制**: 内置拥塞控制算法
- **流量控制**: 防止接收方过载

## 故障排除

### 常见问题

1. **连接失败**
   - 检查服务器是否启动
   - 确认防火墙设置
   - 验证IP地址和端口

2. **消息丢失**
   - 检查网络连接状态
   - 查看控制台错误日志

3. **证书错误**
   - 项目使用自签名证书，这是正常的

### 调试模式

设置环境变量启用详细日志：

```bash
RUST_LOG=debug cargo run -- server
RUST_LOG=debug cargo run -- client --username Alice
```

## 开发计划

- [ ] 文件传输支持
- [ ] 用户认证系统
- [ ] 聊天室/频道功能
- [ ] 消息历史记录
- [ ] Web客户端界面
- [ ] 移动端适配

## 贡献

欢迎提交Issue和Pull Request来改进项目！

## 许可证

MIT License

## 联系方式

如有问题，请提交Issue或联系维护者。