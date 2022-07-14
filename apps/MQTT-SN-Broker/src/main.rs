#![warn(rust_2018_idioms)]


use std::env;
use std::error::Error;
use tokio::net::UdpSocket;
use log::*;
use simplelog::*;
use std::str;
use tokio::time;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::task;


const DEFAULT_MULTICAST_PORT: u16 = 60006;
const DEFAULT_MULTICAST: &str = "239.255.42.98";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

use std::io::{self};


use mqtt_sn_lib::{
    ConnectionDb::ConnectionDb, SubscriberDb::SubscriberDb, TopicDb::TopicDb,
    Transfer::Transfer, Functions::process_input, MTU,BroadcastAdvertise::BroadcastAdvertise
};

use DTLS::dtls_server::DtlsServer;


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


macro_rules! dbg_fn {
    () => {
        $crate::eprintln!("[{}:{}]", function!(), line!());
    };
    ($val:expr $(,)?) => {
        
        match $val {
            tmp => {
                
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
    
    buf: [u8; MTU],
    to_send: Option<(usize, SocketAddr)>,
    connection_db: ConnectionDb,
    subscriber_db: SubscriberDb,
    topic_db: TopicDb,
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
            topic_id_counter: 1,
            input_bytes: Vec::new(),
            size: 0,
        };

        
        loop {
            while let Some(buf) = transfer.egress_buffers.pop() {
                dbg!(buf.clone());
                let (peer, bytes_buf) = buf;
                let amt = socket.send_to(&bytes_buf[..], &peer).await?;
                info!("send_to {} to {}", amt, peer);
            }

            
            to_send = Some(socket.recv_from(&mut buf).await?);
            let to_recv =socket.send_to(&buf, &peer).await?;
            
            let mut temp = buf.clone();
            let s = match str::from_utf8(&temp) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            println!("Result : {}",s);
            
            if let Some((size, peer)) = to_send.clone() {
                info!("recv_from: {:?}", peer);
                transfer.input_bytes = buf.to_vec();
                transfer.peer = peer;
                transfer.size = size;
                dbg_buf!(buf, size); 

                process_input(&buf, size, &mut transfer);
            }
        }
    }
}

async fn task_that_takes_a_second() {
    
    time::sleep(time::Duration::from_secs(1)).await
}

async fn task_that_takes_a_second2() {
    
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    init_logging();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:60000".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    dbg!(&socket);
    println!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: [0u8; MTU],
        to_send: None,
        connection_db: ConnectionDb::new("/tmp/exo-sn-db".to_string()).unwrap(),
        subscriber_db: SubscriberDb::new(),
        topic_db: TopicDb::new(),
    };

    let server_address: SocketAddr = "0.0.0.0:61000".parse().unwrap();
    let dtls_server = DtlsServer {
        server_address,
        buf: [0u8; MTU],
        to_send: None,
        connection_db: ConnectionDb::new("/tmp/exo-sn-db3".to_string()).unwrap(),
        subscriber_db: SubscriberDb::new(),
        topic_db: TopicDb::new(),
    };

    let addr = SocketAddrV4::new(IP_ALL.into(), DEFAULT_MULTICAST_PORT);

    let multi_addr = SocketAddrV4::new(
        
        DEFAULT_MULTICAST
            //    .unwrap()
            .parse::<Ipv4Addr>()
            .expect("Invalid IP"),
        DEFAULT_MULTICAST_PORT,
    );

    println!("Starting server on: {}", addr);
    println!("Multicast address: {}\n", multi_addr);

    let std_socket = bind_multicast(&addr, &multi_addr).expect("Failed to bind multicast socket");

    let socket = UdpSocket::from_std(std_socket).unwrap();

    let ss: SocketAddr = multi_addr.into();
    let broadcast_advertise = BroadcastAdvertise {
        socket,
        addr: ss,
        buf: [0u8; MTU], // Ethernet MTU
        to_send: None,
        connection_db: ConnectionDb::new("/tmp/exo-sn-db2".to_string()).unwrap(),
        subscriber_db: SubscriberDb::new(),
        topic_db: TopicDb::new(),
    };

    let resp1 = task::spawn(timing_wheel());
    let resp2 = task::spawn(timing_wheel2());
    
    let broker_thread = task::spawn(server.run());
    let broker_thread2 = task::spawn(dtls_server.run());

    let _ = broker_thread2.await?;
    let _ = broker_thread.await?;
    
    let _ = resp1.await?;
    let _ = resp2.await?;

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
