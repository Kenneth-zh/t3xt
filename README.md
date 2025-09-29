# T3XT - 基于QUIC的服务器间文本消息软件# T3XT - 真正的QUIC P2P即时通信软件



T3XT是一个使用Rust编写的基于QUIC协议的高性能文本消息系统，支持两个服务器实例之间的双向实时通信。T3XT是一个基于QUIC协议的**真正的点对点（P2P）**即时通信软件，使用Rust编写。与传统的客户端-服务器架构不同，T3XT中的每个节点都具有同等地位，既能接受连接也能发起连接，实现真正的去中心化通信。



## ✨ 特性## ✨ 为什么选择P2P架构？



- 🚀 **基于QUIC协议** - 利用现代网络协议的高性能和可靠性- � **平等地位**: 所有节点都是平等的，没有中央服务器

- 💬 **实时双向通信** - 两个服务器可以互相发送文本消息- 🔗 **直接连接**: 消息直接在节点之间传输，减少延迟

- ❤️ **自动心跳机制** - 维持连接稳定性- 💪 **容错性强**: 没有单点故障，网络更稳定

- 🔒 **TLS安全连接** - 使用自签名证书确保连接安全- 🔒 **隐私保护**: 没有中央服务器收集数据

- 🎯 **简洁易用** - 清晰的命令行界面- 📈 **可扩展**: 网络规模可以自然增长



## 🛠️ 安装与构建## 🚀 主要特性



```bash- ⚡ **QUIC协议**: 更快的连接建立和更好的网络性能

# 克隆项目- 🔒 **自动加密**: 内置TLS 1.3加密，确保通信安全  

git clone <your-repo-url>- � **实时消息**: 专注于文本消息的即时传输

cd t3xt- 🌐 **P2P网络**: 真正的去中心化网络拓扑

- � **节点发现**: 通过种子节点自动发现其他节点

# 构建项目- � **心跳检测**: 自动维护网络连接状态

cargo build --release- 📱 **简单易用**: 专为新手设计的简洁界面

```

## 🛠️ 安装和编译

## 📚 使用方法

确保您已安装Rust（1.70+）：

T3XT支持两种运行模式：

```bash

### 服务器模式# 克隆项目

git clone <your-repo>

启动一个服务器等待其他服务器连接：cd t3xt



```bash# 编译项目

cargo run -- --server <服务器ID> --port <端口>cargo build --release

``````



示例：## 📖 使用方法

```bash

cargo run -- --server ServerA --port 10001### 基本使用

```

T3XT现在只有一个统一的命令，每个节点都是平等的：

### 客户端模式

```bash

连接到已有的服务器：# 启动第一个节点（作为种子节点）

./target/release/t3xt start --username Alice --port 8080

```bash

cargo run -- --connect <目标地址> --target-port <目标端口> --client-id <客户端ID># 启动第二个节点，连接到第一个节点

```./target/release/t3xt start --username Bob --port 8081 --peers 127.0.0.1:8080



示例：# 启动第三个节点，可以连接到任何已存在的节点

```bash./target/release/t3xt start --username Carol --port 8082 --peers 127.0.0.1:8080,127.0.0.1:8081

cargo run -- --connect 127.0.0.1 --target-port 10001 --client-id ServerB```

```

## 📋 命令行参数

## 🧪 测试双向通信

```

### 方法1：使用测试脚本T3XT - 基于QUIC的P2P即时通信软件

```bash

./test.shUsage: t3xt start [OPTIONS] --username <USERNAME>

```

Options:

### 方法2：手动测试  -u, --username <USERNAME>  用户名（必需）

  -p, --port <PORT>         本地监听端口 [默认: 8080]  

1. **终端1** - 启动第一个服务器：  -e, --peers <PEERS>       种子节点地址列表，用逗号分隔

   ```bash  -h, --help                显示帮助信息

   cargo run -- --server ServerA --port 10001```

   ```

## 💡 使用示例

2. **终端2** - 启动第二个服务器并连接：

   ```bash### 场景1：本地测试（同一台电脑）

   cargo run -- --connect 127.0.0.1 --target-port 10001 --client-id ServerB

   ```**终端1 - 启动第一个节点：**

```bash

3. **测试通信**：./target/release/t3xt start --username Alice --port 8080

   - 在任一终端输入文本消息并按回车```

   - 消息会实时显示在另一个终端

   - 输入 `/quit` 退出程序**终端2 - 启动第二个节点：**

```bash  

## 📖 命令行参数./target/release/t3xt start --username Bob --port 8081 --peers 127.0.0.1:8080

```

| 参数 | 描述 | 默认值 |

|------|------|--------|**终端3 - 启动第三个节点：**

| `-s, --server <ID>` | 服务器模式：指定服务器ID | - |```bash

| `-p, --port <PORT>` | 服务器监听端口 | 10005 |./target/release/t3xt start --username Carol --port 8082 --peers 127.0.0.1:8080

| `-c, --connect <ADDR>` | 客户端模式：连接目标地址 | - |```

| `--target-port <PORT>` | 客户端连接的目标端口 | 10005 |

| `--client-id <ID>` | 客户端ID | client |### 场景2：局域网聊天

| `-h, --help` | 显示帮助信息 | - |

| `-V, --version` | 显示版本信息 | - |**电脑A（IP: 192.168.1.100）：**

```bash

## 🔧 架构说明./target/release/t3xt start --username Alice --port 8080

```

T3XT采用简化的点对点架构：

**电脑B：**

``````bash  

ServerA (服务器模式)     ServerB (客户端模式)./target/release/t3xt start --username Bob --port 8080 --peers 192.168.1.100:8080

    |                          |```

    |←-------- QUIC 连接 ------>|

    |                          |**电脑C：**

    |←------ 文本消息 --------->|```bash

    |←------ 心跳包 ----------->|./target/release/t3xt start --username Carol --port 8080 --peers 192.168.1.100:8080,192.168.1.101:8080

``````



### 消息类型### 场景3：互联网P2P（需要公网IP或端口转发）



- **Hello**: 连接握手消息```bash

- **Welcome**: 连接确认消息# 节点1（公网服务器）

- **Text**: 文本消息./target/release/t3xt start --username Server --port 8080

- **Ping/Pong**: 心跳机制

# 节点2（家用电脑）  

## 📦 依赖项./target/release/t3xt start --username Home --port 8080 --peers your-server.com:8080



- **Quinn** - QUIC协议实现# 节点3（移动设备）

- **Tokio** - 异步运行时./target/release/t3xt start --username Mobile --port 8080 --peers your-server.com:8080

- **Rustls** - TLS安全连接```

- **Serde** - 序列化/反序列化

- **Anyhow** - 错误处理## 🎮 聊天操作

- **Clap** - 命令行解析

连接成功后，您可以：

## 🤝 贡献

- ✏️ **发送消息**: 直接输入文本并按回车

欢迎提交问题和改进建议！- 👋 **查看加入**: 当新节点连接时会显示通知  

- 💔 **查看离开**: 当节点断开时会显示通知

## 📄 许可证- ❌ **退出聊天**: 输入 `quit` 命令退出



本项目采用MIT许可证。## 🔧 技术架构

### P2P网络设计

```
     Alice ←→ Bob
       ↑       ↑  
       ↓       ↓
    Carol ←→ David

每个节点都能：
✅ 监听端口接受连接
✅ 主动连接其他节点  
✅ 转发消息到整个网络
✅ 维护邻居节点列表
```

### 核心组件

1. **统一节点（peer.rs）**: 集成了客户端和服务器功能
2. **消息系统（message.rs）**: 简化的文本消息协议
3. **加密层（crypto.rs）**: 自动TLS证书生成
4. **网络发现**: 基于种子节点的网络引导

### 消息类型

- `Text(String)`: 文本消息
- `Join{username, peer_id}`: 节点加入通知  
- `Leave{username, peer_id}`: 节点离开通知
- `Ping/Pong`: 心跳检测

## 🌟 QUIC P2P的优势

相比传统的客户端-服务器架构：

1. **🚫 无单点故障**: 没有中央服务器，任何节点故障都不影响整个网络
2. **⚡ 更低延迟**: 消息在节点间直接传输，无需经过中央服务器
3. **🔒 更好隐私**: 没有中央机构收集或存储你的消息
4. **💰 零运营成本**: 不需要维护昂贵的中央服务器
5. **📈 自然扩展**: 节点越多，网络越强大

## 🛡️ 安全特性

- **自动加密**: 所有连接都使用TLS 1.3自动加密
- **唯一标识**: 每个节点都有唯一的UUID标识
- **消息完整性**: 确保消息在传输中不被篡改
- **连接验证**: 只有有效的QUIC连接才能传输消息

## 🎯 设计理念

T3XT专为**新手用户**设计：

- ✅ **功能专注**: 只专注于文本消息，避免功能过载
- ✅ **简单易懂**: 统一的P2P架构，无需理解复杂的客户端-服务器概念  
- ✅ **即插即用**: 提供种子节点地址就能加入网络
- ✅ **零配置**: 自动处理证书生成和网络配置

## 📊 性能优化

- **QUIC优势**: 0-RTT连接重建，多路复用，内置拥塞控制
- **异步IO**: 基于tokio的高性能异步运行时
- **内存效率**: 消息直接转发，不储存历史记录
- **网络优化**: 智能的心跳机制维护连接活跃度

## 🔍 故障排除

### 常见问题

1. **连接失败**
   ```
   错误：Failed to connect to peer
   解决：检查种子节点地址和端口是否正确，防火墙是否开放
   ```

2. **端口冲突**  
   ```
   错误：Address already in use
   解决：更换其他端口，或者关闭占用端口的程序
   ```

3. **没有收到消息**
   ```
   原因：网络分割或节点连接问题
   解决：重启节点，或添加更多种子节点
   ```

### 调试模式

启用详细日志查看网络活动：

```bash
RUST_LOG=debug ./target/release/t3xt start --username Debug --port 8080
```

## 🚀 未来扩展（可选）

基础的P2P文本聊天已经完整实现，如果需要更多功能：

- 📁 **文件传输**: 点对点文件共享
- 🔐 **身份验证**: 基于密钥的身份验证
- 🌍 **DHT网络**: 更智能的节点发现
- 📱 **移动端**: 手机APP适配
- 🖥️ **图形界面**: GUI客户端

## 📞 联系和贡献

这是一个为新手设计的简单P2P聊天程序。如果您有任何问题或建议，欢迎提交Issue！

---

**享受真正的P2P通信体验！** 🎉