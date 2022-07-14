use anyhow::Result;
use clap::{App, AppSettings, Arg};
use exofense_dtls::udp_conn::Conn;
use exofense_dtls::{config::*, conn::DTLSConn};
use hub::utilities::Error;
use std::fs::File;
use std::io::{BufReader, Write};
use std::sync::Arc;
use tokio::net::UdpSocket;

// cargo run --example dial_verify -- --server 127.0.0.1:4444

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} [{}] {} - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                chrono::Local::now().format("%H:%M:%S.%6f"),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Trace)
        .init();

    let mut app = App::new("DTLS Client")
        .version("0.1.0")
        .author("Rain Liu <yliu@webrtc.rs>")
        .about("An example of DTLS Client")
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::with_name("FULLHELP")
                .help("Prints more detailed help information")
                .long("fullhelp"),
        )
        .arg(
            Arg::with_name("server")
                .required_unless("FULLHELP")
                .takes_value(true)
                .default_value("127.0.0.1:4444")
                .long("server")
                .help("DTLS Server name."),
        );

    let matches = app.clone().get_matches();

    if matches.is_present("FULLHELP") {
        app.print_long_help().unwrap();
        std::process::exit(0);
    }

    let server = matches.value_of("server").unwrap();

    let conn = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    conn.connect(server).await?;
    println!("connecting {}..", server);

    let certificate = hub::utilities::load_key_and_certificate(
        "examples/certificates/client.pem.private_key.pem".into(),
        "examples/certificates/client.pub.pem".into(),
    )?;

    let mut cert_pool = rustls::RootCertStore::empty();
    let f = File::open("examples/certificates/server.pub.pem")?;
    let mut reader = BufReader::new(f);
    if let Err(_) = cert_pool.add_pem_file(&mut reader) {
        return Err(Error::new("cert_pool add_pem_file failed".to_owned()).into());
    }

    let config = Config {
        certificates: vec![certificate],
        extended_master_secret: ExtendedMasterSecretType::Require,
        roots_cas: cert_pool,
        server_name: "webrtc.rs".to_owned(),
        ..Default::default()
    };
    let dtls_conn: Arc<dyn Conn + Send + Sync> =
        Arc::new(DTLSConn::new(conn, config, true, None, None).await?);

    println!("Connected; type 'exit' to shutdown gracefully");
    let _ = hub::utilities::chat(Arc::clone(&dtls_conn)).await;

    dtls_conn.close().await?;

    Ok(())
}
