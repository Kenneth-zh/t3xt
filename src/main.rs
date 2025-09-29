use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod crypto;
mod message;
mod server;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动服务器模式
    Serve {
        /// 服务器ID
        #[arg(short, long, default_value = "Server")]
        id: String,
        
        /// 监听端口
        #[arg(short, long, default_value = "10005")]
        port: u16,
    },
    /// 启动客户端模式（连接到服务器）
    Run {
        /// 目标服务器地址
        #[arg(short, long, default_value = "127.0.0.1")]
        target: String,
        
        /// 目标服务器端口
        #[arg(short, long, default_value = "10005")]
        port: u16,
        
        /// 客户端ID
        #[arg(short, long, default_value = "Client")]
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Serve { id, port } => {
            println!("server started [{}] 监听端口: {}", id, port);
            
            let server = server::Server::new(id, port)?;
            
            if let Err(e) = server.run().await {
                eprintln!("服务器错误: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run { target, port, id } => {
            println!("启动T3XT客户端 [{}] 连接到: {}:{}", id, target, port);
            
            let mut client = client::Client::new(id)?;
            
            if let Err(e) = client.connect(&target, port).await {
                eprintln!("连接失败: {}", e);
                std::process::exit(1);
            }
            
            if let Err(e) = client.run_interactive().await {
                eprintln!("客户端错误: {}", e);
            }
            
            let _ = client.disconnect().await;
        }
    }
    
    Ok(())
}