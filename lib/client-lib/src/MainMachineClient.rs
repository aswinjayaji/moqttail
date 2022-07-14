use crate::ConAck::ConAck;
use crate::Connect::Connect;
use crate::Disconnect::Disconnect;
use crate::MsgType::MsgType;
use crate::PingReq::PingReq;
use crate::PingResp::PingResp;
use crate::Publish::Publish;
use crate::RegAck::RegAck;
use crate::PubAck::PubAck;
use crate::PubRel::PubRel;
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
use crate::WillTopicReq::WillTopicReq;
use crate::WillMsgReq::WillMsgReq;
use crate::MessageDb::{
   MessageDb,MessageDbKey,MessageDbValue
};
use serde::{Deserialize, Serialize};
use rust_fsm::*;
use log::*;
use bytes::BytesMut;
use mqtt_sn_lib::MTU;
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

    

    DISCONNECTED => {
        CONNACK => verify_connack ? ACTIVE[REGISTER] : DISCONNECTED[MSG_TYPE_ERR],
        WILLTOPICREQ => verify_will_topic_req ? WILLSETUP[WILLTOPIC] : DISCONNECTED[MSG_TYPE_ERR],
    },

    
    WILLSETUP => {
        WILLMSGREQ => verify_will_msg_req ? ACTIVE[WILLMSG] : DISCONNECTED[MSG_TYPE_ERR],
    },

    ACTIVE => {
        CONNACK => verify_connack ? ACTIVE[REGISTER] : DISCONNECTED[MSG_TYPE_ERR],
        REGACK => verify_regack ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR],
        PUBACK => verify_puback ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR],

        
        REGISTER => verify_register ? ACTIVE[REGACK] : ACTIVE[REGACK],

        PUBLISH => verify_publish ? ACTIVE[PUBACK] : ACTIVE[PUBACK],

      
        PUBREL => verify_pub_rel ? ACTIVE[PUBCOMP] : ACTIVE[MSG_TYPE_ERR],
        SUBACK => verify_suback ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR], 
        UNSUBACK => verify_unsuback ? ACTIVE[MSG_TYPE_ERR] : ACTIVE[MSG_TYPE_ERR], 
        PINGREQ => verify_ping_req ? ACTIVE[PINGRESP] : ACTIVE[MSG_TYPE_ERR],

        
        PINGRESP => verify_ping_resp ? ACTIVE[DISCONNECT] : ACTIVE[MSG_TYPE_ERR], 

        
        DISCONNECT => verify_disconnect ? ASLEEP[PINGREQ] : LOST[CONNECT], 

        WILLTOPICRESP => verify_will_topic_resp ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR], 
        WILLMSGRESP => verify_will_msg_resp ? ACTIVE[PUBLISH] : ACTIVE[MSG_TYPE_ERR], 
    },

    LOST => {
        CONNACK => verify_connack? ACTIVE[REGISTER] : LOST[CONNECT],
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

fn verify_will_msg_req(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_msg_req, _read_len) = WillMsgReq::try_read(&buf, size, input.into()).unwrap();
    dbg!(will_msg_req.clone());
    match will_msg_req.msg_type {
        0x08 => true, 
        _ => {
            error!(
                "Unexpected MsgType with WillMsgReq. MsgType: {}",
                will_msg_req.msg_type
            );
            false
        }
    }
}

fn verify_will_topic_req(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_topic_req, _read_len) = WillTopicReq::try_read(&buf, size, input.into()).unwrap();
    dbg!(will_topic_req.clone());
    match will_topic_req.msg_type {
        0x06 => true, 
        _ => {
            error!(
                "Unexpected MsgType with WillTopicReq. MsgType: {}",
                will_topic_req.msg_type
            );
            false
        }
    }
}

fn verify_will_topic_resp(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_topic_resp, _read_len) = WillTopicResp::try_read(&buf, size, input.into()).unwrap();
    dbg!(will_topic_resp);
    match will_topic_resp.return_code {
        0 => true,
        1..=3 => {
            println!(
                "WillTopicUpd rejection. ReturnCode: {}",
                will_topic_resp.return_code
            );
            false
        }
        _ => {
            error!(
                "WillTopicUpd rejection. Invalid Return Code: {}",
                will_topic_resp.return_code
            );
            false
        }
    }
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
fn verify_will_msg_resp(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (will_msg_resp, _read_len) = WillMsgResp::try_read(&buf, size, input.into()).unwrap();
    dbg!(will_msg_resp.clone());
    match will_msg_resp.msg_type {
        0x1D => match will_msg_resp.return_code {
            0x00 => true,
            0x01..=0x03 => {
                println!(
                    "WillMsgResp Rejection Return Code: {}",
                    will_msg_resp.return_code
                );
                false
            }
            _ => {
                error!(
                    "WillMsgResp unexpected Return Code: {}",
                    will_msg_resp.return_code
                );
                false
            }
        },
        _ => {
            error!(
                "Wrong MsgType with WillMsgResp. MsgType: {}",
                will_msg_resp.msg_type
            );
            false
        }
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
fn verify_ping_resp(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (ping_resp, _read_len) = PingResp::try_read(&buf, size, input.into()).unwrap();
    dbg!(ping_resp.clone());
    match ping_resp.msg_type {
        0x17 => {
            true 
        }
        _ => {
            error!(
                "Wrong MsgType for PingResp. MsgType: {}",
                ping_resp.msg_type
            );
            false
        }
    }
}
fn verify_connack(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    
    dbg!(input);
    let input = MsgType::CONNACK;
    
    let (conn_ack, _read_len) = ConAck::try_read(buf, size, input.into()).unwrap();
    
    dbg!(conn_ack.clone());

    match conn_ack.msg_type {
        0x05 => match conn_ack.return_code {
            0x00 => {
                
                true
            }
            0x01..=0x03 => {
                error!("CONNACK Rejection: {}", conn_ack.return_code);
                false
            }
            _ => {
                error!("Unknown return code");
                false
            }
        },
        _ => {
            error!(
                "Unexpected message type for CONNACK. Received MsgType: {}",
                conn_ack.msg_type
            );
            false
        }
    }
}


fn verify_regack(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (reg_ack, _read_len) = RegAck::try_read(&buf, size, input.into()).unwrap();
    dbg!(reg_ack.clone());
    match reg_ack.msg_type {
        0x0B => {
            match reg_ack.return_code {
                0x00 => {
                    
                    transfer.topic_id_counter = reg_ack.topic_id;
                    let publish = Publish {
                        len: ("hello".len() + 7) as u8,
                        msg_type: MsgType::REGACK as u8,
                        flags: 0b00000000,
                        topic_id: transfer.topic_id_counter,
                        msg_id: 23,
                        data: "hello".to_string(),
                    };
                    true
                }
                0x01..=0x03 => {
                    error!("RegAck Rejection: {}", reg_ack.return_code);
                    false
                }
                _ => {
                    error!("Unknown return code from REGACK message");
                    error!("RegAck Rejection ReturnCode: {}", reg_ack.return_code);
                    false
                }
            }
        }
        _ => {
            error!(
                "Unexpected message type for RegAck. Received MsgType: {}",
                reg_ack.return_code
            );
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
fn verify_unsuback(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (unsuback, _read_len) = UnsubAck::try_read(&buf, size, input.into()).unwrap();
    dbg!(unsuback.clone());
    match unsuback.msg_type {
        0x15 => {
            true 
        }
        _ => {
            error!(
                "Unexpected MsgType for UnsubAck. MsgType {}",
                unsuback.msg_type
            );
            false
        }
    }
}
fn verify_suback(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (suback, _read_len) = SubAck::try_read(&buf, size, input.into()).unwrap();
    dbg!(suback.clone());
    
    true
}

fn verify_puback(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (pub_ack, _read_len) = PubAck::try_read(&buf, size, input.into()).unwrap();
    dbg!(pub_ack.clone());
    match pub_ack.msg_type {
        0x0D => {
            match pub_ack.return_code {
                0x00 => {
                    true 
                }
                0x01..=0x03 => {
                    println!("PubAck RejectionCode: {}", pub_ack.return_code);
                    false
                }
                _ => {
                    error!("Unexpected PubAck Return Code: {}", pub_ack.return_code);
                    false
                }
            }
        }
        _ => {
            error!(
                "Wrong MsgType for PubAck message. MsgType: {}",
                pub_ack.return_code
            );
            false
        }
    }
}
fn verify_pub_rel(
    _state: StateEnum,
    input: MsgType,
    transfer: &mut Transfer,
    buf: &[u8],
    size: usize,
) -> bool {
    let (pub_rel, _read_len) = PubRel::try_read(&buf, size, input.into()).unwrap();
    dbg!(pub_rel.clone());
    match pub_rel.msg_type {
        0x10 => {
            true 
        }
        _ => {
            error!("Wrong MsgType for PubRel");
            false
        }
    }
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
    let msg_key = MessageDbKey {
        topic_id: publish.topic_id,
    };
    let msg_val = MessageDbValue {
        message: publish.data.clone(),
    };

    transfer.message_db.upsert(msg_key, msg_val);
    let subscribers = transfer.subscriber_db.get(publish.topic_id);
    dbg!(subscribers.clone());
    dbg!(transfer.message_db.get(msg_key));

    match subscribers {
        Some(subs) => {
            
            let mut bytes_buf = BytesMut::with_capacity(MTU);
            publish.try_write(&mut bytes_buf);
            for (sub_socket_addr, _) in subs.peers {
                transfer
                    .egress_buffers
                    .push((sub_socket_addr, bytes_buf.clone()));
            }
            true
        }
        None => false,
    }
}
