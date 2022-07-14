use anyhow::Result;
use clap::{App, AppSettings, Arg};
//use std::io::Write;
use std::str;
use std::sync::Arc;
use tokio::signal;
//use util_conn::conn::*;
use exofense_dtls::{
    config::Config,
    connection_db::ConnectionDb, //state::{State,SerializedState},
    crypto::Certificate,
    extension::extension_use_srtp::SrtpProtectionProfile,
    listener::listen,
    udp_conn::*,
};
// use std::net::SocketAddr;

// cargo run --color=always --package exofense-dtls --example dtls_server -- --host 0.0.0.0:5678

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

    let db = Arc::new(ConnectionDb::new("DB".to_string()).unwrap());
    let mut app = App::new("DTLS Server")
        .version("0.1.0")
        .author("Rain Liu <yliu@webrtc.rs>")
        .about("An example of DTLS Server")
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::with_name("FULLHELP")
                .help("Prints more detailed help information")
                .long("fullhelp"),
        )
        .arg(
            Arg::with_name("host")
                .required_unless("FULLHELP")
                .takes_value(true)
                .long("host")
                .help("DTLS host name."),
        );

    let matches = app.clone().get_matches();

    if matches.is_present("FULLHELP") {
        app.print_long_help().unwrap();
        std::process::exit(0);
    }

    let host = matches.value_of("host").unwrap();
    let cfg = Config {
        certificates: vec![Certificate::generate_self_signed(vec![
            "localhost".to_owned()
        ])?],
        srtp_protection_profiles: vec![SrtpProtectionProfile::Srtp_Aes128_Cm_Hmac_Sha1_80],
        //db:Arc::new(Some(ConnectionDb::new("DB".to_string()).unwrap())),
        ..Default::default()
    };
    println!("Listening on {}", host);
    // println!("Ctrl-C to exit...");

    let listener = Arc::new(listen(host, cfg, Some(Arc::clone(&db))).await?);

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                break;
            }
            result = listener.accept() => {
                if let Ok((dtls_conn, _, _)) = result {
                    // if initial_state.is_none() {
                    //     println!("New connection from client {}", raddr);
                    // }
                    tokio::spawn(async move {
                        let mut buf = [0; 1024];
                        let mut remote_addr = None;
                        while let Ok((n,raddr)) = dtls_conn.recv_from(&mut buf).await{
                            let client_msg = str::from_utf8(&buf[..n])?;
                            println!("{} -> {}", raddr, client_msg);

                            remote_addr = Some(raddr);
                            let message = format!("Echo: {}", client_msg);
                            dtls_conn.send(message.as_bytes()).await?;
                        }
                        println!("closing dtls_conn from {:?}", remote_addr);
                        dtls_conn.close().await
                    });
                }
            }
        }
    }

    listener.close().await
}
