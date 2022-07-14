#[cfg(test)]
mod fragment_buffer_test;

use crate::content::*;
use crate::error::*;
use crate::handshake::handshake_header::*;
use crate::record_layer::record_layer_header::*;

use anyhow::Result;
use std::collections::HashMap;
use std::io::{BufWriter, Cursor};

pub(crate) struct Fragment {
    record_layer_header: RecordLayerHeader,
    handshake_header: HandshakeHeader,
    data: Vec<u8>,
}

pub(crate) struct FragmentBuffer {
    // map of MessageSequenceNumbers that hold slices of fragments
    cache: HashMap<u16, Vec<Fragment>>,

    current_message_sequence_number: u16,
}

impl FragmentBuffer {
    pub fn new() -> Self {
        FragmentBuffer {
            cache: HashMap::new(),
            current_message_sequence_number: 0,
        }
    }

    // Attempts to push a DTLS packet to the FragmentBuffer
    // when it returns true it means the FragmentBuffer has inserted and the buffer shouldn't be handled
    // when an error returns it is fatal, and the DTLS connection should be stopped
    pub fn push(&mut self, mut buf: &[u8]) -> Result<bool> {
        let mut reader = Cursor::new(buf);
        let record_layer_header = RecordLayerHeader::unmarshal(&mut reader)?;

        // Fragment isn't a handshake, we don't need to handle it
        if record_layer_header.content_type != ContentType::Handshake {
            return Ok(false);
        }

        buf = &buf[RECORD_LAYER_HEADER_SIZE..];
        while !buf.is_empty() {
            let mut reader = Cursor::new(buf);
            let handshake_header = HandshakeHeader::unmarshal(&mut reader)?;

            self.cache
                .entry(handshake_header.message_sequence)
                .or_insert_with(Vec::new);

            // end index should be the length of handshake header but if the handshake
            // was fragmented, we should keep them all
            let mut end = HANDSHAKE_HEADER_LENGTH + handshake_header.length as usize;
            if end > buf.len() {
                end = buf.len();
            }

            // Discard all headers, when rebuilding the packet we will re-build
            let data = buf[HANDSHAKE_HEADER_LENGTH..end].to_vec();

            if let Some(x) = self.cache.get_mut(&handshake_header.message_sequence) {
                x.push(Fragment {
                    record_layer_header,
                    handshake_header,
                    data,
                });
            }
            buf = &buf[end..];
        }

        Ok(true)
    }

    pub fn pop(&mut self) -> Result<(Vec<u8>, u16)> {
        let seq_num = self.current_message_sequence_number;
        if !self.cache.contains_key(&seq_num) {
            return Err(Error::ErrEmptyFragment.into());
        }

        let (content, epoch) = if let Some(frags) = self.cache.get_mut(&seq_num) {
            let mut raw_message = vec![];
            // Recursively collect up
            if !append_message(0, frags, &mut raw_message) {
                return Err(Error::ErrEmptyFragment.into());
            }

            let mut first_header = frags[0].handshake_header;
            first_header.fragment_offset = 0;
            first_header.fragment_length = first_header.length;

            let mut raw_header = vec![];
            {
                let mut writer = BufWriter::<&mut Vec<u8>>::new(raw_header.as_mut());
                if first_header.marshal(&mut writer).is_err() {
                    return Err(Error::ErrEmptyFragment.into());
                }
            }

            let message_epoch = frags[0].record_layer_header.epoch;

            raw_header.extend_from_slice(&raw_message);

            (raw_header, message_epoch)
        } else {
            return Err(Error::ErrEmptyFragment.into());
        };

        self.cache.remove(&seq_num);
        self.current_message_sequence_number += 1;

        Ok((content, epoch))
    }
}

fn append_message(target_offset: u32, frags: &[Fragment], raw_message: &mut Vec<u8>) -> bool {
    for f in frags {
        if f.handshake_header.fragment_offset == target_offset {
            let fragment_end =
                f.handshake_header.fragment_offset + f.handshake_header.fragment_length;
            if fragment_end != f.handshake_header.length
                && !append_message(fragment_end, frags, raw_message)
            {
                return false;
            }

            let mut message = vec![];
            message.extend_from_slice(&f.data);
            message.extend_from_slice(raw_message);
            *raw_message = message;
            return true;
        }
    }

    false
}
