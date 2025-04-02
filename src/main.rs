use clap::{Args as ClapArgs, Parser, Subcommand};
use color_eyre::Result;

mod commands;
mod jas;

#[derive(Debug, Subcommand)]
enum Command {
    /// Register a new ticket by token URI and save ticket data to a JSON file
    #[command(arg_required_else_help = true)]
    Register {
        /// Path to save the ticket data
        #[arg(short, long)]
        file_path: String,
        /// Token URI to register
        uri: String,
    },
    /// Get current authentication requests
    #[command(arg_required_else_help = true)]
    Update {
        /// Path to ticket file
        #[arg(short, long)]
        file_path: String,
    },
    /// Accept an authentication requests
    #[command(arg_required_else_help = true)]
    Accept {
        /// Path to ticket file
        #[arg(short, long)]
        file_path: String,
        #[command(flatten)]
        args: AcceptArgs,
    },
}

#[derive(Debug, ClapArgs)]
#[group(required = true, multiple = false)]
struct AcceptArgs {
    /// Session ID from the authentication request received with `update` command
    session_id: Option<String>,
    /// Run in daemon mode and automatically accept all requests
    #[arg(short, long)]
    daemon: bool,
}

#[derive(Debug, Parser)]
#[command(name = env!("CARGO_BIN_NAME"))]
#[command(about = "CLI to manage Aladdin 2FA tickets", long_about = None)]
struct Args {
    /// CA certificate to verify server against
    #[arg(short, long)]
    ca_cert_path: Option<String>,
    /// Retry count for failed requests
    #[arg(short, long)]
    retries: Option<u8>,
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    match args.command {
        Command::Register { file_path, uri } => {
            commands::register(&uri, &file_path, args.ca_cert_path.as_deref(), args.retries)
                .await?;
        }
        Command::Update { file_path } => {
            commands::update(&file_path, args.ca_cert_path.as_deref(), args.retries).await?;
        }
        Command::Accept {
            file_path,
            args: AcceptArgs { daemon, session_id },
        } => {
            if let Some(session_id) = session_id {
                commands::accept(
                    &file_path,
                    &session_id,
                    args.ca_cert_path.as_deref(),
                    args.retries,
                )
                .await?;
            } else if daemon {
                commands::run_daemon(&file_path, args.ca_cert_path.as_deref(), args.retries)
                    .await?;
            } else {
                unreachable!("`session_id` and `daemon` arguments are mutually exclusive");
            }
        }
    }

    Ok(())
}
