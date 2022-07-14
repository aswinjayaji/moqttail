use crate::ConAck::ConAck;
use crate::Connect::Connect;
use crate::Disconnect::Disconnect;
use crate::MsgType::MsgType;
use crate::PingReq::PingReq;
use crate::PingResp::PingResp;
use crate::Publish::Publish;
use crate::RegAck::RegAck;
use crate::Register::Register;
use crate::StateEnum::StateEnum;
use crate::SubAck::SubAck;
use crate::Subscribe::Subscribe;
use crate::Transfer::Transfer;
use crate::UnsubAck::UnsubAck;
use crate::Unsubscribe::Unsubscribe;
use crate::WillMsg::WillMsg;
use crate::WillMsgResp::WillMsgResp;
use crate::WillMsgUpd::WillMsgUpd;
use crate::WillTopic::WillTopic;
use crate::WillTopicResp::WillTopicResp;
use crate::WillTopicUpd::WillTopicUpd;
use crate::MTU;
use crate::Flags::{QoS::QoS,TopicIdType::TopicIdType};

use log::*;
use rust_fsm::*;
use serde::{Deserialize, Serialize};

use bytes::BytesMut;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MainMachine {
    pub machine: rust_fsm::StateMachine<MQTT_SN>,
}


state_machine! {
    input_enum(MsgType)
    state_enum(StateEnum)
    output_enum(MsgType)
    transfer_struct(Transfer)
    derive(Serialize, Deserialize, Clone, Debug, PartialEq)
    pub MQTT_SN(DISCONNECTED)

    ACTIVE => {
        PUBLISH => verify_publish ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR],
        PINGREQ => verify_ping_req ? ACTIVE[SUBACK] : ACTIVE[MSG_TYPE_ERR],
        WILLTOPIC => verify_will_topic ? ACTIVE[MSG_TYPE_ERR] : ACTIVE[MSG_TYPE_ERR],
        WILLMSG => verify_will_msg ? ACTIVE[MSG_TYPE_ERR] : ACTIVE[MSG_TYPE_ERR],
        SUBSCRIBE => verify_subscribe ? ACTIVE[SUBACK] : ACTIVE[MSG_TYPE_ERR],
        REGISTER => verify_register ? ACTIVE[SUBACK] : ACTIVE[MSG_TYPE_ERR],
        UNSUBSCRIBE => verify_unsubscribe ? ACTIVE[SUBACK] : ACTIVE[MSG_TYPE_ERR],
        DISCONNECT => verify_disconnect ? ASLEEP[DISCONNECT] : ACTIVE[MSG_TYPE_ERR],
        WILLTOPICUPD => verify_will_topic_update ? ACTIVE[WILLTOPICRESP] : ACTIVE[MSG_TYPE_ERR],
        WILLMSGUPD => verify_will_msg_update ? ACTIVE[WILLMSGRESP] : ACTIVE[MSG_TYPE_ERR],
    },

    DISCONNECTED => {
        CONNECT => verify_connect ? ACTIVE[CONNACK] : DISCONNECTED[MSG_TYPE_ERR],
    },
}

fn verify_publish(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (publish, _read_len) = Publish::try_read(&buf, size, input.into()).unwrap();
    // dbg_buf!(buf, size);
    dbg!(publish.clone());

    let subscribers = transfer.subscriber_db.get(publish.topic_id);
    let tmpsub = subscribers.clone();
    println!("Subscribed Client list:");
    match tmpsub {
        Some(sub) => {
            for key in sub.peers.keys(){
                println!("{}",key);
            }
            ()
        }
        None => (),
    }

    match subscribers {
        Some(subs) => {
            let mut bytes_buf = BytesMut::with_capacity(MTU);
            publish.try_write(&mut bytes_buf);
            for (sub_socket_addr, _) in subs.peers {
                transfer.egress_buffers.push((sub_socket_addr, bytes_buf.clone()));
            }
            true
        }
        None => false,
    }
}

fn verify_ping_req(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (ping_req, _read_len) = PingReq::try_read(&buf, size, input.into()).unwrap();
    dbg!(ping_req.clone());
    let ping_resp = PingResp {
        len: 2,
        msg_type: MsgType::PINGRESP as u8,
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    ping_resp.try_write(&mut bytes_buf);
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    true
}

fn verify_connect(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (conn, _read_len) = Connect::try_read(buf, size, input.into()).unwrap();
    dbg!(conn.clone());
    let con_ack = ConAck {
        len: 3,
        msg_type: MsgType::CONNACK as u8,
        return_code: 0,
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    con_ack.try_write(&mut bytes_buf);
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    dbg!(con_ack.clone());
    true
}

fn verify_will_msg_update(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_msg_update, _read_len) = WillMsgUpd::try_read(&buf, size, input.into()).unwrap();
    dbg!(will_msg_update.clone());
    let will_msg_resp = WillMsgResp {
        len: 3,
        msg_type: MsgType::WILLMSGRESP as u8,
        return_code: 0, 
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    will_msg_resp.try_write(&mut bytes_buf);
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    true
}

fn verify_will_topic_update(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_topic_update, _read_len) = WillTopicUpd::try_read(&buf, size, input.into()).unwrap();
    dbg!(will_topic_update.clone());
    let will_topic_resp = WillTopicResp {
        len: 3,
        msg_type: MsgType::WILLTOPICRESP as u8,
        return_code: 0, 
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    will_topic_resp.try_write(&mut bytes_buf);
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    true
}

fn verify_unsubscribe(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (unsub, _read_len) = Unsubscribe::try_read(&buf, size, input.into()).unwrap();
    dbg!(unsub.clone());
    match transfer.topic_db.get(&unsub.topic_name) {
        Some(topic_id) => {
            match transfer.subscriber_db.delete(topic_id, transfer.peer) {
                Some(_id) => {
                    let unsub_ack = UnsubAck {
                        len: 8,
                        msg_type: MsgType::UNSUBACK as u8,
                        msg_id: unsub.msg_id,
                    };
                    dbg!(unsub_ack.clone());
                    let mut bytes_buf = BytesMut::with_capacity(MTU);
                    unsub_ack.try_write(&mut bytes_buf);
                    transfer.egress_buffers.push((transfer.peer, bytes_buf));
                    true
                }
                None => {
                    error!(
                        "unsubscribe topic id and peer not found: {} {:?}",
                        topic_id, transfer.peer
                    );
                    false
                }
            }
        }
        None => {
            error!("unsubscribe topic name not found: {}", unsub.topic_name);
            false
        }
    }
}

fn verify_register(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (register, _read_len) = Register::try_read(&buf, size, input.into()).unwrap();
    dbg!(register.clone());
    match transfer.topic_db.get(&register.topic_name) {
        Some(id) => {
            let reg_ack = RegAck {
                len: 7,
                msg_type: MsgType::REGACK as u8,
                topic_id: id,
                msg_id: register.msg_id,
                return_code: 0, 
            };
            dbg!(reg_ack.clone());

            let mut bytes_buf = BytesMut::with_capacity(MTU);
            reg_ack.try_write(&mut bytes_buf);
            transfer.egress_buffers.push((transfer.peer, bytes_buf));
            true
        }
        None => {
            error!(
                "Register: topic name doesn't exist, {:?}",
                register.topic_name
            );
            false
        }
    }
}

fn verify_disconnect(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (disconnect, _read_len) = Disconnect::try_read(&buf, size, input.into()).unwrap();
    dbg!(disconnect.clone());
    let new_disconnect = Disconnect {
        len: 2,
        msg_type: MsgType::DISCONNECT as u8,
        duration: 8,
    };
    let mut bytes_buf = BytesMut::with_capacity(MTU);
    new_disconnect.try_write(&mut bytes_buf);
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    true
}

fn verify_subscribe(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (sub, _read_len) = Subscribe::try_read(&buf, size, input.into()).unwrap();
    dbg!(sub.clone());
    let topic_id = transfer
        .topic_db
        .create(&sub.topic_name, transfer.topic_id_counter);
    if topic_id == transfer.topic_id_counter {
        transfer.topic_id_counter += 1;
    }
    transfer.subscriber_db.insert(topic_id, transfer.peer, 1);
    let sub_ack = SubAck {
        len: 8,
        msg_type: MsgType::SUBACK as u8,
        flags: 0b101111, 
        topic_id: topic_id,
        msg_id: sub.msg_id,
        return_code: 0, 
    };
    dbg!(sub_ack.clone());

    let mut bytes_buf = BytesMut::with_capacity(MTU);
    sub_ack.try_write(&mut bytes_buf);
    
    transfer.egress_buffers.push((transfer.peer, bytes_buf));
    
   if size != sub.len as usize {
    error!("verify_subscribe: size({}) != len({}).", size, sub.len);
    return false;
}

let dup = (sub.flags & 0b1000_0000) == 1;
dbg!(dup);

let QoS = QoS::new(sub.flags);
dbg!(QoS);

let topic_id_type = TopicIdType::new(sub.flags);
dbg!(topic_id_type);
    true
}

fn verify_will_msg(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_msg, _read_len) = WillMsg::try_read(&buf, size, input.into()).unwrap();
    dbg!(will_msg.clone());

    true
}

fn verify_will_topic(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_topic, _read_len) = WillTopic::try_read(&buf, size, input.into()).unwrap();
    dbg!(will_topic.clone());

    true
}
