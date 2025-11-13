use std::{
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use ssh2::Session;

use crate::{
    model::{AuthMethod, RemoteTarget},
    security::{self, HostCheck},
};

const DEFAULT_SSH_PORT: u16 = 22;
const CONNECT_TIMEOUT_SECS: u64 = 5;

pub fn test_connection(target: &RemoteTarget) -> Result<()> {
    let _ = establish_session(target)?;
    Ok(())
}

pub fn establish_session(target: &RemoteTarget) -> Result<Session> {
    let (host, port) = split_host_port(&target.host);
    let addr = format!("{host}:{port}");
    let socket_addr = resolve_addr(&addr)?.ok_or_else(|| anyhow!("unable to resolve {host}"))?;

    let stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .with_context(|| format!("failed to connect to {addr}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(CONNECT_TIMEOUT_SECS)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(CONNECT_TIMEOUT_SECS)))
        .ok();

    let mut session = Session::new().context("failed to create SSH session")?;
    session.set_tcp_stream(stream);
    session.handshake().context("SSH handshake failed")?;

    if let Some((raw_key, _)) = session.host_key() {
        let fingerprint = security::fingerprint_from_raw(raw_key);
        match security::verify_host(&host, &fingerprint)? {
            HostCheck::Match | HostCheck::New => {}
            HostCheck::Mismatch { expected, got } => {
                return Err(anyhow!(
                    "host key mismatch for {host}. expected {expected}, got {got}"
                ));
            }
        }
    }

    match &target.auth {
        AuthMethod::Password { secret, .. } => session
            .userauth_password(&target.username, secret.as_str())
            .context("authentication failed")?,
        AuthMethod::SshKey {
            private_key,
            passphrase,
            ..
        } => session
            .userauth_pubkey_file(
                &target.username,
                None,
                private_key,
                passphrase.as_deref(),
            )
            .context("public key authentication failed")?,
    };

    if !session.authenticated() {
        return Err(anyhow!("authentication rejected"));
    }

    Ok(session)
}

fn resolve_addr(addr: &str) -> Result<Option<std::net::SocketAddr>> {
    let mut addrs = addr.to_socket_addrs()?;
    Ok(addrs.next())
}

fn split_host_port(host: &str) -> (String, u16) {
    if let Some(rest) = host.strip_prefix('[') {
        if let Some((addr, port)) = rest.split_once("]:") {
            if let Ok(port) = port.parse::<u16>() {
                return (addr.to_string(), port);
            }
        }
        return (host.to_string(), DEFAULT_SSH_PORT);
    }

    if host.matches(':').count() > 1 {
        return (host.to_string(), DEFAULT_SSH_PORT);
    }

    if let Some((name, port_str)) = host.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            return (name.to_string(), port);
        }
    }
    (host.to_string(), DEFAULT_SSH_PORT)
}
