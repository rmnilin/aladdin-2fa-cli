use std::str::FromStr;

use clap::{ArgAction, Args as ClapArgs, Parser, Subcommand as ClapSubcommand};
use color_eyre::Result;
use tracing_subscriber::EnvFilter;

mod jas;
mod subcommands;

#[derive(Debug, Parser)]
#[command(name = env!("CARGO_BIN_NAME"), version, about, long_about = None)]
struct Args {
    /// CA certificate to verify server against
    #[arg(short, long)]
    ca_cert_path: Option<String>,

    /// Retry count for failed requests
    #[arg(short, long)]
    retries: Option<u8>,

    /// Logging level
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, ClapSubcommand)]
enum Subcommand {
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
        flatten_args: AcceptFlattenArgs,
    },
}

#[derive(Debug, ClapArgs)]
#[group(required = true, multiple = false)]
struct AcceptFlattenArgs {
    /// Session ID from the authentication request received with `update` command
    session_id: Option<String>,

    /// Run in daemon mode and automatically accept all requests
    #[arg(short, long)]
    daemon: bool,
}

fn init_tracing(verbose: u8) -> Result<()> {
    use tracing::Level;
    use tracing_subscriber::{fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt};

    let level = match verbose {
        0 => Level::INFO.as_str(),
        1 => Level::DEBUG.as_str(),
        _ => Level::TRACE.as_str(),
    };

    let dependencies_level = match verbose {
        0..=1 => "OFF",
        2 => Level::ERROR.as_str(),
        3 => Level::WARN.as_str(),
        4 => Level::INFO.as_str(),
        5 => Level::DEBUG.as_str(),
        _ => Level::TRACE.as_str(),
    };

    let format_layer = layer().pretty();

    #[cfg(not(feature = "err-loc"))]
    let format_layer = format_layer
        .compact()
        .with_target(false)
        .with_line_number(false)
        .with_file(false);

    registry()
        .with(EnvFilter::from_str(
            format!(
                "{},{}::subcommands={}",
                dependencies_level,
                env!("CARGO_CRATE_NAME"),
                level
            )
            .as_str(),
        )?)
        .with(format_layer)
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let Args {
        subcommand,
        ca_cert_path,
        retries,
        verbose,
    } = Args::parse();

    init_tracing(verbose)?;

    match subcommand {
        Subcommand::Register { file_path, uri } => {
            subcommands::register(&uri, &file_path, ca_cert_path.as_deref(), retries).await?;
        }
        Subcommand::Update { file_path } => {
            subcommands::update(&file_path, ca_cert_path.as_deref(), retries).await?;
        }
        Subcommand::Accept {
            file_path,
            flatten_args: AcceptFlattenArgs { session_id, daemon },
        } => {
            if let Some(session_id) = session_id {
                subcommands::accept(&file_path, &session_id, ca_cert_path.as_deref(), retries)
                    .await?;
            } else if daemon {
                subcommands::run_daemon(&file_path, ca_cert_path.as_deref(), retries).await?;
            } else {
                unreachable!("`session_id` and `daemon` arguments are mutually exclusive");
            }
        }
    }

    Ok(())
}
