#![warn(rust_2018_idioms)]

use log::*;
use simplelog::*;
use std::env;
use std::error::Error;
use std::str;

use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::task;

use std::net::SocketAddr;

use nanoid::nanoid;

const MTU: usize = 1500; 
const LOCAL_IP: &str = "127.0.0.1";



use std::io::{self};

use client_lib::{
    // ConnectionDb::ConnectionDb,
    // SubscriberDb::SubscriberDb,
    // Advertise::Advertise,
    // TopicDb::TopicDb,
    // MessageDb::MessageDb,
    Functions::{
        // process_input,
        // verify_suback2, 
        // subscribe,
        connect,
        verify_connack2, 
        publish},
    // Subscribe::Subscribe,
    // Publish::Publish,
    // MsgType::MsgType,
};
// macro_rules! function {
//     () => {{
//         fn f() {}
//         fn type_name_of<T>(_: T) -> &'static str {
//             std::any::type_name::<T>()
//         }
//         let name = type_name_of(f);
//         &name[..name.len() - 3]
//     }};
// }

// macro_rules! dbg_buf {
//     ($buf:ident, $size:ident) => {
//         let mut i: usize = 0;
//         eprint!("[{}:{}] ", function!(), line!());
//         while i < $size {
//             eprint!("{:#04X?} ", $buf[i]);
//             i += 1;
//         }
//         eprintln!("");
//     };
// }




struct Server {
    socket: UdpSocket,
    buf: [u8; MTU],
    to_send: Option<(usize, SocketAddr)>,
}

fn generate_client_id() -> String {
    format!("moqttail/{}", nanoid!())
}


impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut to_send,
        } = self;

        // let peer: SocketAddr = "127.0.0.1:80"
        //     .parse()
        //     .expect("Unable to parse socket address");

        let arc_socket = Arc::new(&socket);
        let clone_socket = Arc::clone(&arc_socket);
        let client_id = generate_client_id();
        let buf2 = connect(&clone_socket, client_id);
        dbg!(buf2.clone());
        clone_socket.send(&buf2).await?;

        to_send = Some(socket.recv_from(&mut buf).await?);
        if let Some((size, peer)) = to_send.clone() {
            match verify_connack2(&buf, size) {
                Ok(_) => {
                    info!("recv_from: {:?}", peer);
                },
                Err(why) => error!("ConnAck {:?}", why),
            }

            info!("recv_from: {:?}", peer);

        }
        // let buf2 = subscribe("humidity".to_string(),15);
        // clone_socket.send(&buf2).await?;
        
        // to_send = Some(socket.recv_from(&mut buf).await?);
        // if let Some((size, peer)) = to_send.clone() {

        //     dbg_fn!(verify_suback2(&buf, size));
        //     info!("recv_from: {:?}", peer);

        // }
        // loop{
        //     to_send = Some(socket.recv_from(&mut buf).await?);

        //     let s = match str::from_utf8(&buf) {
        //         Ok(v) => v,
        //         Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        //     };
        //     println!("standard : {}",s);
        // } 

        let buf2 = publish(1,25, "sending to temp client".to_string(), 1);
        clone_socket.send(&buf2).await?;
        
        Ok(())
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    init_logging();
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("{}:60000", LOCAL_IP).into())
        .parse()?;
    let local_addr: SocketAddr = if remote_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    }
    .parse()?;

    let socket = UdpSocket::bind(local_addr).await?;
    socket.connect(&remote_addr).await?;

    let server = Server {
        socket,
        buf: [0u8; MTU],
        to_send: None,
    };


    let broker_thread = task::spawn(server.run());
    let _ = broker_thread.await?;

    Ok(())
}

fn init_logging() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
