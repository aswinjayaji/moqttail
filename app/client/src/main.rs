#![warn(rust_2018_idioms)]

use log::*;
use simplelog::*;
use std::env;
use std::error::Error;
use std::mem;
#[macro_use]
extern crate arrayref;
use bytes::{BufMut, BytesMut};
use std::str;

use tokio::net::UdpSocket;
// use tokio::sync::mpsc;
use tokio::time;
use tokio::{ task};

use std::net::{ SocketAddr, SocketAddrV4};

// use clap::{App, Arg};
// use tokio::prelude::*;

// const DEFAULT_MULTICAST_PORT: u16 = 60006;
// const DEFAULT_MULTICAST: &str = "239.255.42.98";
//const MTU: usize = 1500; // for Ethernet
// const IP_ALL: [u8; 4] = [0, 0, 0, 0];
const LOCAL_IP: &str = "172.19.114.226";

// Boardcast every n minutes
static BROADCAST_INTERVAL: u8 = 8;


use std::io::{self};


// use DTLS::dtls_client::DtlsClient;
use client_lib::{
    ConnectionDb::ConnectionDb, SubscriberDb::SubscriberDb,
    Advertise::Advertise,
    Transfer::Transfer,MTU,
    TopicDb::TopicDb,MessageDb::MessageDb,Functions::{process_input,connect},Subscribe::Subscribe,Publish::Publish,MsgType::MsgType,
};


// TODO move to utility lib
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

macro_rules! dbg_buf {
    ($buf:ident, $size:ident) => {
        let mut i: usize = 0;
        eprint!("[{}:{}] ", function!(), line!());
        while i < $size {
            eprint!("{:#04X?} ", $buf[i]);
            i += 1;
        }
        eprintln!("");
    };
}

// dbg macro that prints function name instead of file name.
// https://stackoverflow.com/questions/65946195/understanding-the-dbg-macro-in-rust
macro_rules! dbg_fn {
    () => {
        $crate::eprintln!("[{}:{}]", function!(), line!());
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                // replace file!() with function!()
                eprintln!("[{}:{}] {} = {:#?}",
                    function!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($dbg_fn!($val)),+,)
    };
}



struct Server {
    socket: UdpSocket,
    // buf: Vec<u8>,
    buf: [u8; MTU],
    to_send: Option<(usize, SocketAddr)>,
    connection_db: ConnectionDb,
    subscriber_db: SubscriberDb,
    topic_db: TopicDb,
    message_db: MessageDb,
}

struct BroadcastAdvertise {
    socket: UdpSocket,
    addr: SocketAddr,
    // buf: Vec<u8>,
    buf: [u8; MTU],
    to_send: Option<(usize, SocketAddr)>,
    connection_db: ConnectionDb,
    subscriber_db: SubscriberDb,
    topic_db: TopicDb,
    message_db: MessageDb,
}


impl BroadcastAdvertise {
    async fn run(self) -> Result<(), io::Error> {
        let BroadcastAdvertise {
            socket,
            addr,
            mut buf,
            mut to_send,
            connection_db,
            subscriber_db,
            topic_db,
            message_db,
        } = self;

        let peer: SocketAddr = "127.0.0.1:80"
            .parse()
            .expect("Unable to parse socket address");
        let mut transfer = Transfer {
            peer,
            egress_buffers: Vec::new(),
            subscriber_db: subscriber_db.clone(),
            connection_db: connection_db.clone(),
            topic_db: topic_db.clone(),
            message_db: message_db.clone(),
            topic_id_counter: 0,
            input_bytes: Vec::new(),
            size: 0,
        };

        let advertise = Advertise {
            len: 5,
            msg_type: MsgType::ADVERTISE as u8,
            gw_id: 9,
            duration: BROADCAST_INTERVAL as u16,
        };
        dbg!(advertise.clone());
        let advertise_len = advertise.len;

        let mut bytes_buf = BytesMut::with_capacity(MTU);
        advertise.try_write(&mut bytes_buf);
        let mut interval = time::interval(time::Duration::from_secs(BROADCAST_INTERVAL as u64));
        loop {
            interval.tick().await;
            let amt = socket.send_to(&bytes_buf[..], &addr).await?;
            if amt == advertise_len as usize {
                info!("broadcast advertise message to {}", addr);
            } else {
                error!("broadcast advertise message length not match: amt: {}", amt);
            }
        }
    }
}

impl Server {
    async fn run(self) -> Result<(), io::Error> {
        let Server {
            socket,
            mut buf,
            mut to_send,
            connection_db,
            subscriber_db,
            topic_db,
            message_db,
        } = self;

        let peer: SocketAddr = "127.0.0.1:80"
            .parse()
            .expect("Unable to parse socket address");
        let mut transfer = Transfer {
            peer,
            egress_buffers: Vec::new(),
            subscriber_db: subscriber_db.clone(),
            connection_db: connection_db.clone(),
            topic_db: topic_db.clone(),
            message_db: message_db.clone(),
            topic_id_counter: 1,
            input_bytes: Vec::new(),
            size: 0,
        };

        let arc_socket = Arc::new(&socket);
        let clone_socket = Arc::clone(&arc_socket);

        let buf2 = connect(&clone_socket);
        clone_socket.send(&buf2).await?;
        // dbg!(&socket);

        //let buf2 = subscribe("hello".to_string());
        //clone_socket.send(&buf2).await?;

        // buffer for sending
        // butes_buff:BytesMut, for flexibility, functions can write multiple times,
        // clear it etc...
        loop {
            while let Some(buf) = transfer.egress_buffers.pop() {
                dbg!(buf.clone());
                let (peer, bytes_buf) = buf;
                clone_socket.send(&bytes_buf).await?;
                //let amt = socket.send_to(&bytes_buf[..], &peer).await?;
                //info!("send_to {} to {}", amt, peer);
                info!("Sent to {}", peer);
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            to_send = Some(socket.recv_from(&mut buf).await?);
            if let Some((size, peer)) = to_send.clone() {
                info!("recv_from: {:?}", peer);
                transfer.input_bytes = buf.to_vec();
                transfer.peer = peer;
                transfer.size = size;
                process_input(&buf, size, &mut transfer);
            }
        }
    }
}

async fn task_that_takes_a_second() {
    // println!("task_that_takes_a_second");
    time::sleep(time::Duration::from_secs(1)).await
}

async fn task_that_takes_a_second2() {
    // println!("task_that_takes_a_second2");
    time::sleep(time::Duration::from_secs(1)).await
}

async fn timing_wheel() {
    let mut interval = time::interval(time::Duration::from_secs(2));
    for _i in 0..10000 {
        interval.tick().await;
        task_that_takes_a_second().await;
    }
}

async fn timing_wheel2() {
    let mut interval = time::interval(time::Duration::from_secs(2));
    for _i in 0..10000 {
        interval.tick().await;
        task_that_takes_a_second2().await;
    }
}

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
fn bind_multicast(
    addr: &SocketAddrV4,
    multi_addr: &SocketAddrV4,
) -> Result<std::net::UdpSocket, Box<dyn Error>> {
    use socket2::{Domain, Protocol, Socket, Type};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_multicast_loop_v4(true)?;
    socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

    Ok(socket.into_udp_socket())
}

fn publish(topic: u16, message: String) -> BytesMut {
    let msg_len = message.len() + 7;
    let publish = Publish {
        len: msg_len as u8,
        msg_type: MsgType::PUBLISH as u8,
        flags: 0b00000100,
        topic_id: 2,
        msg_id: 0,
        data: message,
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(publish.clone());
    publish.try_write(&mut bytes_buf);
    bytes_buf
}

fn subscribe(topic: String) -> BytesMut {
    let msg_len = topic.len() + 5;
    let subscribe = Subscribe {
        len: msg_len as u8,
        msg_type: MsgType::SUBSCRIBE as u8,
        flags: 0b00100100,
        msg_id: 0,
        topic_name: topic, // TODO use enum for topic_name or topic_id
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(subscribe.clone());
    subscribe.try_write(&mut bytes_buf);
    bytes_buf
}

use std::sync::{Arc, Mutex};
use std::thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    /*
        let words:Vec<u32> = Arc::new(Mutex::new(vec![]));
        let mut threads = vec![];
        for x in 0..5 {
            threads.push(thread::spawn({
                let clone = Arc::clone(&words);
                move || {
                    let mut v = clone.lock().unwrap();
                    // v.push(x.to_string());
                    v.push(x);
                }
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
        let clone = Arc::clone(&words);
        dbg!(clone.clone());
        let mut v = clone.lock().unwrap();
        dbg!(v.clone());
        dbg!(v.pop());
        println!("{:?}", words);
    */
    init_logging();

    //    let (mut tx, mut rx) = mpsc::channel(32);
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("{}:60000", LOCAL_IP).into())
        .parse()?;

    // We use port 0 to let the operating system allocate an available port for us.
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
        connection_db: ConnectionDb::new("connection-db".to_string()).unwrap(),
        subscriber_db: SubscriberDb::new(),
        topic_db: TopicDb::new(),
        message_db: MessageDb::new("message-db".to_string()).unwrap(),
    };

    let server_address: SocketAddr = format!("{}:61000", LOCAL_IP).parse().unwrap();
    // let dtls_client = DtlsClient {
    //     server_address,
    //     buf: [0u8; MTU],
    //     to_send: None,
    //     connection_db: ConnectionDb::new("connection-db2".to_string()).unwrap(),
    //     subscriber_db: SubscriberDb::new(),
    //     topic_db: TopicDb::new(),
    //     //message_db: MessageDb::new("message-db".to_string()).unwrap(),
    // };

    //let buf = publish(1, "message 1".to_string());
    // TODO socket.send(&buf).await?;

    let broker_thread = task::spawn(server.run());
    // let broker_thread2 = task::spawn(dtls_client.run());
    
    // let _ = broker_thread2.await?;
    let _ = broker_thread.await?;

    /*
    let addr = SocketAddrV4::new(IP_ALL.into(), DEFAULT_MULTICAST_PORT);

    let multi_addr = SocketAddrV4::new(
        // matches.value_of("ip")
        DEFAULT_MULTICAST
        //    .unwrap()
        .parse::<Ipv4Addr>()
        .expect("Invalid IP"),
        DEFAULT_MULTICAST_PORT,
        );

    println!("Starting server on: {}", addr);
    println!("Multicast address: {}\n", multi_addr);

    let std_socket = bind_multicast(&addr, &multi_addr)
        .expect("Failed to bind multicast socket");

    let socket = UdpSocket::from_std(std_socket).unwrap();

    let ss:SocketAddr = multi_addr.into();
    let broadcast_advertise = BroadcastAdvertise {
        socket,
        addr: ss,
        buf: [0u8; MTU], // Ethernet MTU
        to_send: None,
        connection_db: ConnectionDb::new("mqtt-sn-db2".to_string()).unwrap(),
        subscriber_db:  SubscriberDb::new(),
        topic_db:  TopicDb::new(),
        message_db: MessageDb::new("message-db2".to_string()).unwrap(),
        // message_db: server.message_db,
    };

    let timing_wheel_task1 = task::spawn(timing_wheel());
    let timing_wheel_task2 = task::spawn(timing_wheel2());
    let broadcast_advertise_thread = task::spawn(broadcast_advertise.run());
    // This starts the server task.
    let broker_thread = task::spawn(server.run());

    let _ = broker_thread.await?;
    let _ = broadcast_advertise_thread.await?;
    let _ = timing_wheel_task1.await?;
    let _ = timing_wheel_task2.await?;

    */
    Ok(())
}

fn init_logging() {
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
