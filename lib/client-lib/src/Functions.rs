use crate::Connect::Connect;
use crate::ConAck::ConAck;
use crate::SubAck::SubAck;
use crate::PubAck::PubAck;
use crate::Publish::Publish;
use crate::Subscribe::Subscribe;

use crate::MainMachineClient::MainMachine;
use crate::MsgType::MsgType;
use crate::Transfer::Transfer;
use crate::MTU;

use bytes::BytesMut;
use log::*;
use num_traits::FromPrimitive;
use rust_fsm::*;
use std::mem;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use std::net::SocketAddr;

/// Client Error
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Len Error: {0} (expeted {1})")]
    LenError(usize, usize),
    #[error("Wrong Message Type: {0} (expect {1}")]
    WrongMessageType(u8, u8),

    // return code
    #[error("Congestion: {0}")]
    Congestion(u8),
    #[error("Invalid Topic Id: {0}")]
    InvalidTopicId(u8),
    #[error("Not Supported: {0}")]
    NotSupported(u8),
    #[error("Return Code Reserved: {0}")]
    Reserved(u8),
}


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


pub fn process_input(buf: &[u8], size: usize, transfer: &mut Transfer) -> Option<u8> {
    let mut offset = 0;
    let len: u8 = buf[offset];
    // if len != size, ignore the packet
    if size != len as usize {
        error!("size({}) != len({}).", size, len);
        return None;
    }
    dbg_buf!(buf, size);
    offset += mem::size_of::<u8>();
    let msg_type_u8 = buf[offset];
    let msg_type = FromPrimitive::from_u8(msg_type_u8);
    match transfer.connection_db.read(transfer.peer) {
        Some(old_machine) => {
            dbg!(old_machine.clone());
            let mut new_machine = old_machine.clone();
            let _ = new_machine
                .machine
                .consume(&msg_type.unwrap(), transfer, &buf, size);
            // TODO check for return value
            // if return error, clear the egress_buffer
            transfer
                .connection_db
                .update(transfer.peer, &old_machine, &new_machine);
            dbg!(old_machine.clone());
            dbg!(new_machine.machine.state());

            let state = new_machine.machine.state();
            // Some(1)
            // ()
        }
        None => {
            // packet without state machine
            dbg!(buf[1]);
            match FromPrimitive::from_u8(buf[1]) {
                Some(MsgType::CONNACK) => {
                    let mut new_machine = MainMachine {
                        machine: StateMachine::new(),
                    };
                    let _ = new_machine
                        .machine
                        .consume(&msg_type.unwrap(), transfer, &buf, size);
                    // TODO check for return value
                    transfer.connection_db.create(transfer.peer, &new_machine);
                    dbg!(new_machine);
                },
                Some(MsgType::CONNECT) => {
                    let mut new_machine = MainMachine {
                        machine: StateMachine::new(),
                    };
                    let _ = new_machine
                        .machine
                        .consume(&msg_type.unwrap(), transfer, &buf, size);
                    // TODO check for return value
                    transfer.connection_db.create(transfer.peer, &new_machine);
                    dbg!(new_machine);
                },
                _ => (),
            }
        }
    }
    None
    // Some(MsgType::MsgType::ACTIVE)
}

static PUBACK_LEN:u8 = 7;

pub fn pub_ack(topic_id: u16, msg_id: u16) -> BytesMut {
    let pub_ack = PubAck {
        len: PUBACK_LEN,
        msg_type: MsgType::PUBACK as u8,
        topic_id: topic_id,
        msg_id: msg_id,
        return_code: 0,
    };
    let mut bytes_buf = BytesMut::with_capacity(PUBACK_LEN as usize);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(pub_ack.clone());
    pub_ack.try_write(&mut bytes_buf);
    bytes_buf
}
pub fn publish(topic_id: u16, msg_id: u16, message: String, qos_level: i8) -> BytesMut {
    let flags = set_qos_bits(qos_level);
    let len = message.len() + 7;
    let publish = Publish {
        len: len as u8,
        msg_type: MsgType::PUBLISH as u8,
        flags: flags,
        topic_id: topic_id,
        msg_id: msg_id,
        data: message,
    };
    let mut bytes_buf = BytesMut::with_capacity(len);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(publish.clone());
    publish.try_write(&mut bytes_buf);
    bytes_buf
}

pub fn subscribe(topic_name: String, msg_id: u16) -> BytesMut {
    let len = topic_name.len() + 5;
    let subscribe = Subscribe {
        len: len as u8,
        msg_type: MsgType::SUBSCRIBE as u8,
        flags: 0b00100100,
        msg_id,
        topic_name, // TODO use enum for topic_name or topic_id
    };
    let mut bytes_buf = BytesMut::with_capacity(len);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg!(subscribe.clone());
    subscribe.try_write(&mut bytes_buf);
    bytes_buf
}

pub fn connect2(client_id: String) -> BytesMut {
    let len = client_id.len() + 6;
    let connect = Connect {
        len: len as u8,
        msg_type: MsgType::CONNECT as u8,
        flags: 0b00000100,
        protocol_id: 1,
        duration: 30,
        client_id: client_id,
    };
    let mut bytes_buf = BytesMut::with_capacity(len);
    // serialize the con_ack struct into byte(u8) array for the network.
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg_fn!(connect.clone());
    connect.try_write(&mut bytes_buf);
    dbg_fn!(bytes_buf.clone());
    // return false of error, and set egree_buffers to empty.
    bytes_buf
}

pub fn connect(socket: &UdpSocket, client_id: String) -> BytesMut {
    let len = client_id.len() + 6;
    let connect = Connect {
        len: len as u8,
        msg_type: MsgType::CONNECT as u8,
        flags: 0b00000100,
        protocol_id: 1,
        duration: 30,
        client_id: client_id,
    };
    let mut bytes_buf = BytesMut::with_capacity(len);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg_fn!(connect.clone());
    connect.try_write(&mut bytes_buf);
    dbg_fn!(bytes_buf.clone());
    // return false of error, and set egree_buffers to empty.
    // let amt = socket.send(&bytes_buf[..]);
    bytes_buf
}

fn return_code(val: u8) -> Result<(), ClientError> {
    match val {
        0 => {
            Ok(())
        },
        1 => {
            Err(ClientError::Congestion(val))
        },
        2 => {
            Err(ClientError::InvalidTopicId(val))
        },
        3 => {
            Err(ClientError::NotSupported(val))
        },
        _ => {
            Err(ClientError::Reserved(val))
        },
    }
}

// CONACK: len(0) MsgType(1) ReturnCode(2)
// expect:     3          5
pub fn verify_connack2(
    buf: &[u8],
    size: usize,
) -> Result<(), ClientError> {
    // deserialize from u8 array to the Conn structure.
    let msg_type = MsgType::CONNACK;
    dbg_fn!(msg_type);
    let (conn_ack, read_len) = ConAck::try_read(buf, size, msg_type.into()).unwrap();
    dbg_fn!(conn_ack.clone());

    // check length
    if (read_len != 3) {
        return Err(ClientError::LenError(read_len, 3));
    }

    // check message type
    match conn_ack.msg_type {
        // check return code
        msg_type => return_code(conn_ack.return_code),
        _ => {
            Err(ClientError::WrongMessageType(conn_ack.return_code, msg_type.into()))
        }
    }
}
pub fn verify_puback2(
    buf: &[u8],
    size: usize,
// ) -> bool {
) -> Result<(u16, u16), ClientError> {
    let msg_type = MsgType::PUBACK;
    let (pub_ack, read_len) = PubAck::try_read(&buf, size, msg_type.into()).unwrap();
    dbg_fn!(pub_ack.clone());

    // check length
    if (read_len != 8) {
        return Err(ClientError::LenError(read_len, 8));
    }

    // check message type
    match pub_ack.msg_type {
        // TODO check flags
        //
        // TODO match msg_id & topic_id
        //
        // check return code
        msg_type => {
            match return_code(pub_ack.return_code) {
                Ok(_) => Ok((pub_ack.topic_id, pub_ack.msg_id)),
                Err(why) => Err(why),
            }
        }
        _ => {
            Err(ClientError::WrongMessageType(pub_ack.return_code, msg_type.into()))
        }
    }
}


pub fn verify_suback2(
    buf: &[u8],
    size: usize,
// ) -> bool {
) -> Result<u16, ClientError> {
    let msg_type = MsgType::CONNACK;
    let (sub_ack, read_len) = SubAck::try_read(&buf, size, msg_type.into()).unwrap();
    dbg_fn!(sub_ack.clone());

    // check length
    if (read_len != 8) {
        return Err(ClientError::LenError(read_len, 8));
    }

    // check message type
    match sub_ack.msg_type {
        // TODO check flags
        //
        // TODO match msg_id & topic_id
        //
        // check return code
        msg_type => {
            match return_code(sub_ack.return_code) {
                Ok(_) => Ok(sub_ack.msg_id),
                Err(why) => Err(why),
            }
        }
        _ => {
            Err(ClientError::WrongMessageType(sub_ack.return_code, msg_type.into()))
        }
    }
}

pub fn verify_publish3(
    channel_tx: &Sender<(Vec<u8>, SocketAddr)>,
    addr: &SocketAddr,
    buf: &[u8],
    size: usize,
) -> Result<(u16,u16, String, BytesMut), ClientError> {
    let msg_type = MsgType::PUBLISH;
    let (publish, read_len) = Publish::try_read(&buf, size, msg_type.into()).unwrap();
    dbg_fn!(publish.clone());
    dbg_fn!((size, read_len));

    // check length
    /*
    if (read_len != size) {
        return Err(ClientError::LenError(read_len, size));
    }
    */

    // check message type
    match publish.msg_type {
        // TODO check flags
        //
        // TODO match msg_id & topic_id
        //
        // check return code
        msg_type => {
            dbg_fn!(publish.flags);
            println!("{:b}", publish.flags);
            println!("{:?}", get_qos_level(publish.flags));
            let mut bytes_buf = BytesMut::with_capacity(MTU);
            match get_qos_level(publish.flags) {
                // send PUCACK for QoS 1 & 2
                1 | 2 => {
                    bytes_buf = pub_ack(publish.topic_id, publish.msg_id);
                    dbg_fn!(&bytes_buf);
                    channel_tx.try_send((bytes_buf[..].to_vec(), *addr));
                }
                _ => {}
            }

            Ok((publish.topic_id, publish.msg_id, publish.data, bytes_buf))
        }
        _ => {
            Err(ClientError::WrongMessageType(publish.msg_type, msg_type.into()))
        }
    }
}

pub fn verify_publish2(
    buf: &[u8],
    size: usize,
) -> Result<(u16,u16, String), ClientError> {
// ) -> Result<(u16,u16, String, BytesMut), ClientError> {
    let msg_type = MsgType::PUBLISH;
    let (publish, read_len) = Publish::try_read(&buf, size, msg_type.into()).unwrap();
    dbg_fn!(publish.clone());
    dbg_fn!((size, read_len));

    // check length
    /*
    if (read_len != size) {
        return Err(ClientError::LenError(read_len, size));
    }
    */

    // check message type
    match publish.msg_type {
        // TODO check flags
        //
        // TODO match msg_id & topic_id
        //
        // check return code
        msg_type => {
            dbg_fn!(publish.flags);
            Ok((publish.topic_id, publish.msg_id, publish.data))
        }
        _ => {
            Err(ClientError::WrongMessageType(publish.msg_type, msg_type.into()))
        }
    }
}


pub fn get_qos_level(qos_bits: u8) -> i8 {
    match qos_bits {
        0 => 0,
        0b00100000 => 1,
        0b01000000 => 2,
        0b01100000 => -1,
        _ => 0,
    }
}

pub fn set_qos_bits(qos_level: i8) -> u8 {
    match qos_level {
        0 => 0,
        1 => 0b00100000,
        2 => 0b01000000,
        -1 => 0b01100000,
        _ => 0,
    }
}


pub fn publish_socket(socket: &UdpSocket, topic_id: u16, msg: String) -> BytesMut {
    let msg_len = msg.len() + 5;
    let publish = Publish {
        len: msg_len as u8,
        msg_type: MsgType::PUBLISH as u8,
        flags: 0b00000100,
        topic_id: topic_id,
        msg_id: 30,
        data: msg.to_string(),
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    // serialize the con_ack struct into byte(u8) array for the network.
    dbg_fn!(publish.clone());
    publish.try_write(&mut bytes_buf);
    dbg_fn!(bytes_buf.clone());
    // return false of error, and set egree_buffers to empty.
    // let amt = socket.send(&bytes_buf[..]);
    bytes_buf
}

