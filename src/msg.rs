#![allow(dead_code)]
use std::fmt;

use thiserror::Error;
use serde::{Serialize, Deserialize};

pub type MessageOpcode = u16;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MessageType {
    ReadRequest,
    WriteRequest,
    Data,
    Acknowledgement,
    Error
}

impl MessageType {
    fn to_opcode(msg_type: Self) -> MessageOpcode {
        match msg_type {
            MessageType::ReadRequest => 0x01,
            MessageType::WriteRequest => 0x02,
            MessageType::Data => 0x03,
            MessageType::Acknowledgement => 0x04,
            MessageType::Error => 0x05
        }
    }

    fn from_opcode(opcode: MessageOpcode) -> Option<Self> {
        match opcode {
            0x01 => Some(MessageType::ReadRequest),
            0x02 => Some(MessageType::WriteRequest),
            0x03 => Some(MessageType::Data),
            0x04 => Some(MessageType::Acknowledgement),
            0x05 => Some(MessageType::Error),
            _ => None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ReadWriteRequestMessageMode {
    NetAscii,
    Octet,
    Mail
}

impl ReadWriteRequestMessageMode {
    fn to_string(mode: Self) -> String {
        match mode {
            ReadWriteRequestMessageMode::NetAscii => "netascii".to_string(),
            ReadWriteRequestMessageMode::Octet => "octet".to_string(),
            ReadWriteRequestMessageMode::Mail => "mail".to_string()
        }
    }

    fn from_string(mode_string: String) ->
    Option<Self> {
        match mode_string.as_str() {
            "netascii" | "NETASCII" | "NetAscii" =>
                Some(ReadWriteRequestMessageMode::NetAscii),
            "octet" | "OCTET" | "Octet" =>
                Some(ReadWriteRequestMessageMode::Octet),
            "mail" | "MAIL" | "Mail" => Some(ReadWriteRequestMessageMode::Mail),
            _ => None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Error)]
pub enum ParseError {
    TooShort,
    TooLong,
    InvalidOpcode,
    NoFilename,
    InvalidFilename,
    NoMode,
    InvalidMode,
    InvalidErrorCode,
    NoErrorMessage,
    InvalidErrorMessage 
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg: &str = match self {
            ParseError::TooShort => "Specified message contains too little \
                 bytes for its message type",
            ParseError::TooLong => "Specified message contains too many bytes \
                 for its message type",
            ParseError::InvalidOpcode => "Specified message has an invalid \
                 opcode (either unknown or mismatched for its message type)",
            ParseError::NoFilename => "Specified message lacks a filename when \
                 it should have one",
            ParseError::InvalidFilename => "Specified message has an invalid \
                 filename (likely improperly terminated or contains forbidden \
                 characters)",
            ParseError::NoMode => "Specified message lacks a mode string when \
                 it should have one",
            ParseError::InvalidMode => "Specified message has an invalid mode \
                 string (likely improperly terminated or contains forbidden \
                 characters)",
            ParseError::InvalidErrorCode => "Specified message has an invalid \
                 error code",
            ParseError::NoErrorMessage => "Specified message lacks an error \
                 message string when it should have one",
            ParseError::InvalidErrorMessage => "Specified message has an  \
                invalid error message string (likely improperly terminated \
                 or contains forbidden characters)"
        };

        write!(f, "{}", msg)
    }
}

pub trait Message {
    fn opcode(&self) -> MessageOpcode;
}

/****************************** READ REQUEST **********************************/

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadRequestMessage {
    msg_type: MessageType,
    filename: String,
    mode: ReadWriteRequestMessageMode
}

impl ReadRequestMessage {
    pub fn new(filename: String, mode: ReadWriteRequestMessageMode) -> Self {
        ReadRequestMessage {
            msg_type: MessageType::ReadRequest,
            filename: filename.clone(),
            mode: mode
        }
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }

    pub fn mode(&self) -> ReadWriteRequestMessageMode {
        self.mode
    }
}

impl Message for ReadRequestMessage {
    fn opcode(&self) -> MessageOpcode {
        MessageType::to_opcode(self.msg_type)
    }
}

/****************************** WRITE REQUEST  ********************************/

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteRequestMessage {
    msg_type: MessageType,
    filename: String,
    mode: ReadWriteRequestMessageMode
}

impl WriteRequestMessage {
    pub fn new(filename: String, mode: ReadWriteRequestMessageMode) -> Self {
        WriteRequestMessage {
            msg_type: MessageType::WriteRequest,
            filename: filename.clone(),
            mode: mode
        }
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }

    pub fn mode(&self) -> ReadWriteRequestMessageMode {
        self.mode
    }
}

impl Message for WriteRequestMessage {
    fn opcode(&self) -> MessageOpcode {
        MessageType::to_opcode(self.msg_type)
    }
}

/*********************************** DATA *************************************/

pub type DataMessageBlockNumber = u16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataMessage {
    msg_type: MessageType,
    block_num: DataMessageBlockNumber,
    data: Vec<u8>
}

impl DataMessage {
    pub fn new(block_num: DataMessageBlockNumber, data: Vec<u8>) -> Self {
        DataMessage {
            msg_type: MessageType::Data,
            block_num: block_num,
            data: data.clone()
        }
    }

    pub fn block_num(&self) -> DataMessageBlockNumber {
        self.block_num
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl Message for DataMessage {
    fn opcode(&self) -> MessageOpcode {
        MessageType::to_opcode(self.msg_type)
    }
}

/****************************** ACKNOWLEDGEMENT *******************************/

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcknowledgementMessage {
    msg_type: MessageType,
    block_num: DataMessageBlockNumber
}

impl AcknowledgementMessage {
    pub fn new(block_num: DataMessageBlockNumber) -> Self {
        AcknowledgementMessage {
            msg_type: MessageType::Acknowledgement,
            block_num: block_num
        }
    }

    pub fn block_num(&self) -> DataMessageBlockNumber {
        self.block_num
    }
}

impl Message for AcknowledgementMessage {
    fn opcode(&self) -> MessageOpcode {
        MessageType::to_opcode(self.msg_type)
    }
}

/********************************** ERROR  ************************************/

pub type ErrorMessageCode = u16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMessage {
    msg_type: MessageType,
    code: ErrorMessageCode,
    message: String
}

impl ErrorMessage {
    pub fn new(code: ErrorMessageCode, message: String) -> Self {
        ErrorMessage {
            msg_type: MessageType::Error,
            code: code,
            message: message.clone()
        }
    }

    pub fn code(&self) -> ErrorMessageCode {
        self.code
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl Message for ErrorMessage {
    fn opcode(&self) -> MessageOpcode {
        MessageType::to_opcode(self.msg_type)
    }
}

