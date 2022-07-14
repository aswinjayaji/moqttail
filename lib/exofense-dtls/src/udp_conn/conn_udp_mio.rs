use super::*;

use crate::mio_udp_socket::MioUdpSocket;

#[async_trait]
impl Conn for MioUdpSocket {
    async fn connect(&self, addr: SocketAddr) -> Result<()> {
        Ok(self.connect(addr).await?)
    }

    async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        Ok(self.recv(buf).await?)
    }

    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        Ok(self.recv_from(buf).await?)
    }

    async fn send(&self, buf: &[u8]) -> Result<usize> {
        Ok(self.send(buf).await?)
    }

    async fn send_to(&self, buf: &[u8], target: SocketAddr) -> Result<usize> {
        Ok(self.send_to(buf, target).await?)
    }

    async fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.local_addr().await?)
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }

    async fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr().await
    }
}
