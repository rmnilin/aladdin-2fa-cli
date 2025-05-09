use std::{fs, time::Duration};

use color_eyre::Result;

use crate::jas::{Client, Ticket};

fn get_ca_cert(path: Option<&str>) -> Result<Option<Vec<u8>>> {
    Ok(match path {
        Some(path) => Some(fs::read(path)?),
        None => None,
    })
}

pub async fn register(
    uri: &str,
    file_path: &str,
    ca_cert_path: Option<&str>,
    retries: Option<u8>,
) -> Result<()> {
    let ca_cert = get_ca_cert(ca_cert_path)?;
    let client = Client::new(ca_cert.as_deref(), retries)?;
    let ticket = client.register(uri).await?;
    ticket.save(file_path)?;
    tracing::info!("Ticket registered and saved to {}", file_path);
    Ok(())
}

pub async fn update(
    file_path: &str,
    ca_cert_path: Option<&str>,
    retries: Option<u8>,
) -> Result<()> {
    let ticket = Ticket::load(file_path)?;
    let ca_cert = get_ca_cert(ca_cert_path)?;
    let client = Client::new(ca_cert.as_deref(), retries)?;
    let session_ids = client.update(&ticket).await?;
    if session_ids.is_empty() {
        tracing::debug!("No authentication requests received");
    } else {
        for session_id in session_ids {
            tracing::info!(
                "Authentication request with session ID {} received",
                session_id
            );
        }
    }
    Ok(())
}

pub async fn accept(
    file_path: &str,
    session_id: &str,
    ca_cert_path: Option<&str>,
    retries: Option<u8>,
) -> Result<()> {
    let ticket = Ticket::load(file_path)?;
    let ca_cert = get_ca_cert(ca_cert_path)?;
    let client = Client::new(ca_cert.as_deref(), retries)?;
    client.accept(&ticket, session_id).await?;
    tracing::info!(
        "Authentication request with session ID {} accepted",
        session_id
    );
    Ok(())
}

async fn run_daemon_iteration(
    ticket: &Ticket,
    ca_cert: Option<&[u8]>,
    retries: Option<u8>,
) -> Result<()> {
    let client = Client::new(ca_cert, retries)?;
    let session_ids = client.update(ticket).await?;
    if session_ids.is_empty() {
        tracing::debug!("No authentication requests received");
    } else {
        for session_id in client.update(ticket).await? {
            tracing::debug!(
                "Authentication request with session ID {} received",
                session_id
            );
            client.accept(ticket, &session_id).await?;
            tracing::info!(
                "Authentication request with session ID {} accepted",
                session_id
            );
        }
    }
    Ok(())
}

pub async fn run_daemon(
    file_path: &str,
    ca_cert_path: Option<&str>,
    retries: Option<u8>,
) -> Result<()> {
    let ticket = Ticket::load(file_path)?;
    let ca_cert = get_ca_cert(ca_cert_path)?;
    loop {
        let result = run_daemon_iteration(&ticket, ca_cert.as_deref(), retries).await;
        if let Err(error) = result {
            tracing::error!("{}", error);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
