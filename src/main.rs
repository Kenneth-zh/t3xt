use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod client;
mod crypto;
mod message;
mod server;

/// T3XT - 基于QUIC的点对点即时通信软件
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动服务器模式
    Server {
        /// 绑定地址
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,
        /// 监听端口
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// 启动客户端模式
    Client {
        /// 用户名
        #[arg(short, long)]
        username: String,
        /// 服务器地址
        #[arg(short, long, default_value = "127.0.0.1")]
        server: String,
        /// 服务器端口
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { bind, port } => {
            info!("Starting T3XT server on {}:{}", bind, port);
            
            let server = server::Server::new(&bind, port)?;
            
            // 启动服务器
            if let Err(e) = server.run().await {
                eprintln!("Server error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Client { username, server: server_addr, port } => {
            info!("Starting T3XT client, connecting to {}:{}", server_addr, port);
            
            let mut client = client::Client::new(username)?;
            
            // 连接到服务器
            if let Err(e) = client.connect(&server_addr, port).await {
                eprintln!("Failed to connect to server: {}", e);
                std::process::exit(1);
            }
            
            // 启动交互式聊天
            if let Err(e) = client.start_interactive_chat().await {
                eprintln!("Chat error: {}", e);
            }
            
            // 断开连接
            if let Err(e) = client.disconnect().await {
                eprintln!("Failed to disconnect: {}", e);
            }
        }
    }

    Ok(())
}
