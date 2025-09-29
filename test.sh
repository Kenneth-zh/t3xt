#!/bin/bash

# T3XT 双向通信测试脚本

echo "🧪 T3XT 双向通信测试"
echo "=================="
echo ""

echo "📋 测试步骤:"
echo "1. 在第一个终端运行: cargo run -- --server ServerA --port 10001"
echo "2. 在第二个终端运行: cargo run -- --connect 127.0.0.1 --target-port 10001 --client-id ServerB"
echo "3. 两个终端都可以互相发送消息"
echo "4. 输入 '/quit' 退出任一程序"
echo ""

echo "🔧 快速测试命令:"
echo "服务器A (端口10001): cargo run -- --server ServerA --port 10001"
echo "服务器B (连接到A):   cargo run -- --connect 127.0.0.1 --target-port 10001 --client-id ServerB"
echo ""

echo "💡 提示:"
echo "- ServerA 会等待连接并可以接收消息"
echo "- ServerB 会连接到ServerA并可以发送/接收消息"
echo "- 双方都可以输入文本进行聊天"
echo "- 支持心跳机制维持连接"
echo ""

read -p "按回车键查看详细使用说明..."
echo ""

echo "📖 详细说明:"
echo ""
echo "参数说明："
echo "--server <ID>      : 服务器模式，指定服务器ID"
echo "--port <PORT>      : 服务器监听端口 (默认: 10005)"
echo "--connect <ADDR>   : 客户端模式，连接目标地址"
echo "--target-port <P>  : 客户端连接的目标端口 (默认: 10005)"
echo "--client-id <ID>   : 客户端ID (默认: client)"
echo ""

echo "🌟 特性："
echo "✅ 基于QUIC协议的高性能通信"
echo "✅ 支持实时双向文本消息"
echo "✅ 自动心跳保持连接"
echo "✅ 优雅的连接管理"
echo "✅ 简洁的命令行界面"
echo ""