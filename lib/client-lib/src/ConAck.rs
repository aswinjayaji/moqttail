use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};

#[derive(Debug, thiserror::Error)]
pub enum ConnAckError {
    #[error("ConnAck Rejection: {0}")]
    ConnAckRejection(u8),
    #[error("ConnAck Unknown Code: {0}")]
    ConnAckUnknownCode(u8),
    #[error("ConnAck Wrong Message Type: {0}")]
    ConnAckWrongMessageType(u8),
}


#[derive(Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct ConAck {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub return_code: u8, 
}

impl ConAck {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(val: &u8) -> Result<(), ConnAckError> {
        match *val {
            0x00 => {
                // XXX Ok(())
                Err(ConnAckError::ConnAckRejection(*val))
            },
            0x01..=0x03 => {
                Err(ConnAckError::ConnAckRejection(*val))
            },
            _ => {
                Err(ConnAckError::ConnAckUnknownCode(*val))
            },
        }
    }
    fn constraint_return_code(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
}
