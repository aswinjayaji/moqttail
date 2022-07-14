use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use std::str;
use crate::MTU;
#[derive(Debug, Clone)]
pub struct MessageDb {
    pub db: sled::Db,
    pub name: String,
    pub old_value: String,
}

#[derive(Debug, Clone, Copy, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct MessageDbKey {
    pub topic_id: u16,
}

impl MessageDbKey {
    fn constraint_topic_id(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }
}

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct MessageDbValue {
    pub message: String,
}


impl MessageDbValue {
    fn constraint_message(_val: &String) -> bool {
        //dbg!(_val);
        true
    }
}

impl MessageDb {
    pub fn new(name: String) -> sled::Result<MessageDb> {
        let db: sled::Db = sled::open(name.clone())?;
        let new_db = MessageDb {
            db,
            name,
            old_value: String::new(),
        };
        
        Ok(new_db)
    }
    pub fn insert(
        &self,
        key: MessageDbKey,
        value: MessageDbValue,
    ) -> sled::Result<Result<(), sled::CompareAndSwapError>> {
        let mut key_buf = BytesMut::with_capacity(MTU);
        let mut value_buf = BytesMut::with_capacity(MTU);

        key.try_write(&mut key_buf);
        value.try_write(&mut value_buf);

        self.db.compare_and_swap(
            &key_buf[..],
            None as Option<&[u8]>,
            Some(&value_buf[..]),  
        )
    }
    pub fn upsert(&self, key: MessageDbKey, value: MessageDbValue) {
        let mut key_buf = BytesMut::with_capacity(MTU);
        let mut value_buf = BytesMut::with_capacity(MTU);
        
        key.try_write(&mut key_buf);
        value.try_write(&mut value_buf);
        let get_result = self.db.get(&key_buf[..]).unwrap();
        match get_result {
            Some(old_value) => {
                
                let result = self.db.compare_and_swap(
                    &key_buf[..],
                    Some(old_value),      
                    Some(&value_buf[..]), 
                );
            }
            None => {
                let result = self.db.compare_and_swap(
                    &key_buf[..],
                    None as Option<&[u8]>, 
                    Some(&value_buf[..]),  
                );
            }
        }
    }

    pub fn get(&self, key: MessageDbKey) -> Option<sled::IVec> {
        let mut key_buf = BytesMut::with_capacity(MTU);
        key.try_write(&mut key_buf);
        match self.db.get(&key_buf[..]).unwrap() {
            Some(bytes) => Some(bytes),
            None => None,
        }
    }

    pub fn delete(&self, key: MessageDbKey) -> sled::Result<Option<sled::IVec>> {
        let mut key_buf = BytesMut::with_capacity(MTU);
        key.try_write(&mut key_buf);
        match self.db.remove(&key_buf[..]).unwrap() {
            Some(bytes) => Ok(Some(bytes)),
            None => Ok(None),
        }
    }
}
