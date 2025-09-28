#!/bin/bash

# T3XT 即时通信软件使用示例

echo "T3XT - QUIC点对点即时通信软件演示"
echo "======================================="
echo

echo "1. 启动服务器（在另一个终端窗口中运行）："
echo "   ./target/release/t3xt server"
echo

echo "2. 启动客户端："
echo "   ./target/release/t3xt client --username Alice"
echo "   ./target/release/t3xt client --username Bob"
echo

echo "3. 在客户端中输入消息并按回车发送"
echo "   输入 'quit' 退出聊天"
echo

echo "4. 测试远程连接："
echo "   服务器： ./target/release/t3xt server --bind 0.0.0.0 --port 8080"
echo "   客户端： ./target/release/t3xt client --username User --server 192.168.1.100 --port 8080"
echo

echo "特性："
echo "- ✅ QUIC协议传输（快速、可靠）"
echo "- ✅ TLS 1.3自动加密"
echo "- ✅ 实时消息传输"
echo "- ✅ 多用户支持"
echo "- ✅ 心跳检测"
echo "- ✅ 命令行界面"
echo

echo "开始体验吧！"