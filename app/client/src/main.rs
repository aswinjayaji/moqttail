use tokio::net::UdpSocket;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let sock = UdpSocket::bind("127.0.0.3:8080").await?;

    let remote_addr = "127.0.0.1:8080";
    sock.connect(remote_addr).await?;
    let mut buf = [0; 1024];
    let str = "hiii";
    loop {
        let len = str.len();
        let len = sock.send(&buf[..len]).await?;
        println!("{:?} bytes sent", len);
        let len = sock.recv(&mut buf).await?;
        println!("{:?} bytes received from {:?}", len, remote_addr);
    }
}