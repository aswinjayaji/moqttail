use anyhow::Result;
use clap::{App, AppSettings, Arg};
//use std::io::Write;
use std::str;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
//use exofense_dtls::udp_conn::Conn;
use exofense_dtls::{
    config::Config, conn::DTLSConn, crypto::Certificate,
    extension::extension_use_srtp::SrtpProtectionProfile, udp_conn::*,
};
use std::io;

async fn create_client(
    ca: Arc<dyn Conn + Send + Sync>,
    mut cfg: Config,
    generate_certificate: bool,
) -> Result<impl Conn> {
    if generate_certificate {
        let client_cert = Certificate::generate_self_signed(vec!["localhost".to_owned()])?;
        cfg.certificates = vec![client_cert];
    }

    cfg.insecure_skip_verify = true;
    DTLSConn::new(ca, cfg, true, None, None).await
}

// cargo run --color=always --package exofense-dtls --example dtls_client -- --server 0.0.0.0:5678

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    /*env_logger::Builder::new()
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
    .init();*/

    let mut app = App::new("SCTP Ping")
        .version("0.1.0")
        .author("Rain Liu <yliu@webrtc.rs>")
        .about("An example of SCTP Client")
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
                .long("server")
                .help("SCTP Server name."),
        );

    let matches = app.clone().get_matches();

    if matches.is_present("FULLHELP") {
        app.print_long_help().unwrap();
        std::process::exit(0);
    }

    let server = matches.value_of("server").unwrap();

    let conn = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    println!("Local address -> {}", conn.local_addr().unwrap());
    conn.connect(server).await?;
    println!("Connecting to server {}", server);

    let cfg = Config {
        srtp_protection_profiles: vec![SrtpProtectionProfile::Srtp_Aes128_Cm_Hmac_Sha1_80],
        ..Default::default()
    };
    let dtls_conn = create_client(conn, cfg, true).await?;

    println!("Connected to server {}", server);

    loop {
        // let message = format!(
        //     "hello world msg {} from dtls client: {}",
        //     i,
        //     dtls_conn.local_addr().await?
        // );
        let mut message = String::new();

        io::stdin()
            .read_line(&mut message)
            .expect("Failed to read line");

        let message = message.trim().as_bytes();
        dtls_conn.send(message).await?;

        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut buf = [0; 1024];
        let n = dtls_conn.recv(&mut buf).await?;
        println!("{}", str::from_utf8(&buf[..n])?);
    }

    // dtls_conn.close().await
}