use clap::{Parser, Subcommand};

mod server;
mod client;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// 启动服务端
    Server { addr: String },
    /// 启动客户端
    Client { addr: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Server { addr } => server::run(addr).await?,
        Mode::Client { addr } => client::run(addr).await?,
    }

    Ok(())
}
