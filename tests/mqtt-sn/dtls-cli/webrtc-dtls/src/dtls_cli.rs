use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct DtlsCli {
    pub target: SocketAddr,
    pub flight1: bool,
    pub times1: Option<usize>,
    pub flight3: bool,
    pub times3: Option<usize>,
    pub cookie: Vec<u8>,
    pub flight5: bool,
    pub times5: Option<usize>,
}

impl Default for DtlsCli {
    fn default() -> Self {
        DtlsCli {
            target: "127.0.0.1:50000".parse().unwrap(),
            flight1: false,
            times1: None,
            flight3: false,
            times3: None,
            cookie: vec![],
            flight5: false,
            times5: None,
        }
    }
}
