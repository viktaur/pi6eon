use std::net::Ipv6Addr;
use clap::Parser;
use clap::Subcommand;
use anyhow::Result;
use crate::tcp::*;

mod tcp;
mod msg;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Action to perform: set up a connection or listen for an incoming one
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    Setup {
        /// Address to start a connection with
        #[arg(short, long)]
        address: Ipv6Addr,
        /// Port to send the connection request to
        #[arg(short, long)]
        port: u16
    },
    Listen {
        /// Port to listen in for incoming connections
        #[arg(short, long)]
        port: u16
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Action::Setup { address, port } => {
            setup_connection(address, port).await?
        },
        Action::Listen { port } => {
            listen_for_connection(port).await?
        },
    };

    println!("Action is {:?}", args.command);

    Ok(())
}
