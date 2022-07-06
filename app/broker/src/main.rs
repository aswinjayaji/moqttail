use tokio::net::UdpSocket;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()>{
    let sock = UdpSocket::bind("127.0.0.1:8080").await?;
    let mut buffer = [0;1024];
    loop{
        let (len,addr) = sock.recv_from(&mut buffer).await?;
        println!("{:?} bytrs recieved from {:?}",len,addr);
        let len = sock.send_to(&buffer[..len],addr).await?;
        println!("{:?} bytes sent",len);
    }
}