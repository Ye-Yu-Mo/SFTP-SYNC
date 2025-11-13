use std::{
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use ssh2::Session;

use crate::model::RemoteTarget;

const DEFAULT_SSH_PORT: u16 = 22;
const CONNECT_TIMEOUT_SECS: u64 = 5;

pub fn test_connection(target: &RemoteTarget) -> Result<()> {
    let (host, port) = split_host_port(&target.host);
    let addr = format!("{host}:{port}");
    let socket_addr = resolve_addr(&addr)?.ok_or_else(|| anyhow!("unable to resolve {host}"))?;

    let stream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(CONNECT_TIMEOUT_SECS))
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

    session
        .userauth_password(&target.username, target.password.as_str())
        .context("authentication failed")?;

    if !session.authenticated() {
        return Err(anyhow!("authentication rejected"));
    }

    Ok(())
}

fn resolve_addr(addr: &str) -> Result<Option<std::net::SocketAddr>> {
    let mut addrs = addr.to_socket_addrs()?;
    Ok(addrs.next())
}

fn split_host_port(host: &str) -> (String, u16) {
    if let Some((name, port_str)) = host.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            return (name.to_string(), port);
        }
    }
    (host.to_string(), DEFAULT_SSH_PORT)
}
