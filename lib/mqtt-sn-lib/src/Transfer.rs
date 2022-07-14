use crate::ConnectionDb;
use crate::SubscriberDb;
use crate::TopicDb;

use bytes::BytesMut;
use std::net::SocketAddr;


#[derive(Debug, Clone)]
pub struct Transfer {
    pub peer: SocketAddr,
    pub topic_id_counter: u16,
    
    pub egress_buffers: Vec<(SocketAddr, BytesMut)>,
    pub subscriber_db: SubscriberDb::SubscriberDb,
    pub connection_db: ConnectionDb::ConnectionDb,
    pub topic_db: TopicDb::TopicDb,
    pub input_bytes: Vec<u8>,
    pub size: usize,
}
