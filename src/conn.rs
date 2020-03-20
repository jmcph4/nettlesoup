#![allow(dead_code)]
use serde::{Serialize, Deserialize};

use crate::msg;

pub type TID = u16;
pub type SequenceNumber = u16;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Connection {
    local_tid: TID,                         /* local TID (source port) */
    remote_tid: TID,                        /* remote TID (destination port) */
    curr_seq: SequenceNumber,               /* current sequence number */
    last_msg: Option<msg::AnyMessage>,      /* latest message */
}

impl Connection {
    pub fn new(local_tid: TID, remote_tid: TID) -> Self {
        Connection {
            local_tid: local_tid,
            remote_tid: remote_tid,
            curr_seq: 0,
            last_msg: None
        }
    }

    pub fn local_tid(&self) -> TID {
        self.local_tid
    }

    pub fn remote_tid(&self) -> TID {
        self.remote_tid
    }

    pub fn curr_seq(&self) -> SequenceNumber {
        self.curr_seq
    }

    pub fn last_msg(&self) -> Option<msg::AnyMessage> {
        self.last_msg.clone()
    }

    pub fn add_msg(&mut self, message: msg::AnyMessage) {
        self.last_msg = Some(message);
        self.curr_seq += 1;
    }
}

