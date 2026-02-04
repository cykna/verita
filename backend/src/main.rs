//! This example demonstrates an HTTP server that serves files from a directory.
//!
//! Checkout the `README.md` for guidance.

use std::{
    ascii, fs,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    str,
    sync::Arc,
};

use color_eyre::eyre::{Context, Report, Result};
use dotenv::dotenv;
use quinn::crypto::rustls::QuicServerConfig;
use rustls::{
    RootCertStore,
    client::WebPkiServerVerifier,
    crypto::CryptoProvider,
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, pem::PemObject},
    server::{WebPkiClientVerifier, danger::ClientCertVerifier},
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    CryptoProvider::install_default(rustls::crypto::ring::default_provider()).unwrap();
    dotenv().ok();
    let (certs, key) = if let (Ok(key_path), Ok(cert_path)) = (
        std::env::var("QUIC_KEY_PATH").map(|v| PathBuf::from(v)),
        std::env::var("QUIC_CERT_PATH").map(|v| PathBuf::from(v)),
    ) {
        let key = if key_path.extension().is_some_and(|x| x == "der") {
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                fs::read(key_path).context("failed to read private key file")?,
            ))
        } else {
            PrivateKeyDer::from_pem_file(key_path)
                .context("failed to read PEM from private key file")?
        };

        let cert_chain = if cert_path.extension().is_some_and(|x| x == "der") {
            vec![CertificateDer::from(
                fs::read(cert_path).context("failed to read certificate chain file")?,
            )]
        } else {
            CertificateDer::pem_file_iter(cert_path)
                .context("failed to read PEM from certificate chain file")?
                .collect::<Result<_, _>>()
                .context("invalid PEM-encoded certificate")?
        };

        (cert_chain, key)
    } else {
        return Err(Report::msg("No key nor pem files provided"));
    };

    let ca_path = std::env::var("QUIC_CA_PATH").context("QUIC_CA_PATH not set")?;
    let ca_file = fs::read(ca_path).context("failed to read CA file")?;
    let mut ca_reader = std::io::BufReader::new(&ca_file[..]);

    let mut root_store = RootCertStore::empty();
    let ca_certs = rustls_pemfile::certs(&mut ca_reader).collect::<Result<Vec<_>, _>>()?;

    for cert in ca_certs {
        root_store.add(cert)?;
    }

    let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_store)).build()?;

    let server_crypto = rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(certs, key)?;

    let mut server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));

    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let addr = SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        std::env::var("SERVER_PORT")?.parse().unwrap(),
    ));
    let endpoint = quinn::Endpoint::server(server_config, addr)?;
    eprintln!("listening on {}", endpoint.local_addr()?);

    while let Some(conn) = endpoint.accept().await {
        // if options
        //     .connection_limit
        //     .is_some_and(|n| endpoint.open_connections() >= n)
        // {
        //     info!("refusing due to open connection limit");
        //     conn.refuse();
        // } else if Some(conn.remote_address()) == options.block {
        //     info!("refusing blocked client IP address");
        //     conn.refuse();
        // } else if options.stateless_retry && !conn.remote_address_validated() {
        //     info!("requiring connection to validate its address");
        //     conn.retry().unwrap();
        // } else {
        info!("accepting connection");
        let fut = handle_connection(conn);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                error!("connection failed: {reason}", reason = e.to_string())
            }
        });
    }

    Ok(())
}

async fn handle_connection(conn: quinn::Incoming) -> Result<()> {
    let connection = conn.await?;

    info!("established");

    // Each stream initiated by the client constitutes a new request.
    loop {
        let stream = connection.accept_bi().await;
        let stream = match stream {
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                info!("connection closed");
                return Ok(());
            }
            Err(e) => {
                return Err(e.into());
            }
            Ok(s) => s,
        };
        let fut = handle_request(stream);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                error!("failed: {reason}", reason = e.to_string());
            }
        });
    }
}

async fn handle_request(
    (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
) -> Result<()> {
    let req = recv
        .read_to_end(64 * 1024)
        .await
        .map_err(|e| Report::msg(format!("failed reading request: {}", e)))?;
    let mut escaped = String::new();
    for &x in &req[..] {
        let part = ascii::escape_default(x).collect::<Vec<_>>();
        escaped.push_str(str::from_utf8(&part).unwrap());
    }
    info!(content = %escaped);
    // Execute the request
    let resp = process_get(&req).unwrap_or_else(|e| {
        error!("failed: {}", e);
        format!("failed to process request: {e}\n").into_bytes()
    });
    // Write the response
    send.write_all(&resp)
        .await
        .map_err(|e| Report::msg(format!("failed to send response: {}", e)))?;
    // Gracefully terminate the stream
    send.finish().unwrap();
    info!("complete");
    Ok(())
}

fn process_get(x: &[u8]) -> Result<Vec<u8>> {
    println!("{x:?}");
    Ok(x.to_vec())
}
