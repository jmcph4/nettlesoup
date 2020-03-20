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
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ParseError> where Self: Sized;
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

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        /* opcode */
        bytes.extend_from_slice(
            &MessageType::to_opcode(self.msg_type).to_be_bytes());
        
        /* filename string */
        for byte in self.filename.bytes() {
            bytes.push(byte);
        }

        bytes.push('\0' as u8); /* null terminate */

        /* request mode */
        for ch in ReadWriteRequestMessageMode::to_string(self.mode).bytes() {
            bytes.push(ch);
        }

        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ParseError> {
        if bytes.len() < 5 { /* bounds check */
            return Err(ParseError::TooShort);
        }

        /* parse opcode */
        let opcode: MessageOpcode = ((bytes[0] as u16) << 8) |
                                            bytes[1] as u16; 

        /* this field is implicit in all message types, but we still need to
            validate the correctness of it in the wire format */
        let msg_type: Option<MessageType> = MessageType::from_opcode(opcode);

        if msg_type.is_none() { /* check for failure of our helper */
            return Err(ParseError::InvalidOpcode);
        }

        /* check the opcode actually matches the message type */
        if msg_type.unwrap() != MessageType::ReadRequest {
            return Err(ParseError::InvalidOpcode);
        }
        
        /* parse filename */
        let mut filename: String = String::new();

        let mut c: usize = 1;
        let mut curr_char: char = bytes[c] as char;

        /* iterate over bytes, grabbing characters until null byte (we can do
            this because of the encoding of TFTP strings) */
        while curr_char != '\0' {
            if c >= bytes.len() { /* bounds check */
                return Err(ParseError::InvalidFilename);
            }

            c += 1;
            curr_char = bytes[c] as char;
            filename.push(curr_char);
        }

        /* parse mode */
        if c >= bytes.len() { /* bounds check */
            return Err(ParseError::NoMode);
        }
       
        /* adjust for null byte */
        if c + 1 < bytes.len() {
            curr_char = bytes[c+1] as char;
        }

        let mut mode_string: String = String::new();
       
        /* iterate over bytes, grabbing characters until null byte (we can do
            this because of the encoding of TFTP strings) */
        while curr_char != '\0' {
            if c >= bytes.len() { /* bounds check */
                if mode_string.len() == 0 {
                    return Err(ParseError::NoMode);
                } else {
                    return Err(ParseError::InvalidMode);
                }
            }

            c += 1;
            curr_char = bytes[c] as char;
            mode_string.push(curr_char);
        }
   
        /* strip trailing null bytes from both filename and mode string */
        filename.pop();
        mode_string.pop();

        if filename.len() == 0 {
            return Err(ParseError::NoFilename);
        }

        if mode_string.len() == 0 {
            return Err(ParseError::NoMode);
        }
 
        let mode: Option<ReadWriteRequestMessageMode> =
            ReadWriteRequestMessageMode::from_string(mode_string);
        
        if mode.is_none() { /* check for failure of our helper */
            return Err(ParseError::InvalidMode);
        }
        
        /* actually construct the message object */ 
        let message: ReadRequestMessage =
            ReadRequestMessage::new(filename, mode.unwrap());

        Ok(message)
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

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        /* opcode */
        bytes.extend_from_slice(
            &MessageType::to_opcode(self.msg_type).to_be_bytes());
        
        /* filename string */
        for byte in self.filename.bytes() {
            bytes.push(byte);
        }

        bytes.push('\0' as u8); /* null terminate */

        /* request mode */
        for ch in ReadWriteRequestMessageMode::to_string(self.mode).bytes() {
            bytes.push(ch);
        }

        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ParseError> {
        if bytes.len() < 5 { /* bounds check */
            return Err(ParseError::TooShort);
        }

        /* parse opcode */
        let opcode: MessageOpcode = ((bytes[0] as u16) << 8) |
                                            bytes[1] as u16; 

        /* this field is implicit in all message types, but we still need to
            validate the correctness of it in the wire format */
        let msg_type: Option<MessageType> = MessageType::from_opcode(opcode);

        if msg_type.is_none() { /* check for failure of our helper */
            return Err(ParseError::InvalidOpcode);
        }

        /* check the opcode actually matches the message type */
        if msg_type.unwrap() != MessageType::WriteRequest {
            return Err(ParseError::InvalidOpcode);
        }
        
        /* parse filename */
        let mut filename: String = String::new();

        let mut c: usize = 1;
        let mut curr_char: char = bytes[c] as char;

        /* iterate over bytes, grabbing characters until null byte (we can do
            this because of the encoding of TFTP strings) */
        while curr_char != '\0' {
            if c >= bytes.len() { /* bounds check */
                return Err(ParseError::InvalidFilename);
            }

            c += 1;
            curr_char = bytes[c] as char;
            filename.push(curr_char);
        }

        /* parse mode */
        if c >= bytes.len() { /* bounds check */
            return Err(ParseError::NoMode);
        }
       
        /* adjust for null byte */
        if c + 1 < bytes.len() {
            curr_char = bytes[c+1] as char;
        }

        let mut mode_string: String = String::new();
       
        /* iterate over bytes, grabbing characters until null byte (we can do
            this because of the encoding of TFTP strings) */
        while curr_char != '\0' {
            if c >= bytes.len() { /* bounds check */
                if mode_string.len() == 0 {
                    return Err(ParseError::NoMode);
                } else {
                    return Err(ParseError::InvalidMode);
                }
            }

            c += 1;
            curr_char = bytes[c] as char;
            mode_string.push(curr_char);
        }
   
        /* strip trailing null bytes from both filename and mode string */
        filename.pop();
        mode_string.pop();

        if filename.len() == 0 {
            return Err(ParseError::NoFilename);
        }

        if mode_string.len() == 0 {
            return Err(ParseError::NoMode);
        }
 
        let mode: Option<ReadWriteRequestMessageMode> =
            ReadWriteRequestMessageMode::from_string(mode_string);
        
        if mode.is_none() { /* check for failure of our helper */
            return Err(ParseError::InvalidMode);
        }
        
        /* actually construct the message object */ 
        let message: WriteRequestMessage =
            WriteRequestMessage::new(filename, mode.unwrap());

        Ok(message)
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

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        /* opcode */
        bytes.extend_from_slice(
            &MessageType::to_opcode(self.msg_type).to_be_bytes());

        /* block number */
        bytes.extend_from_slice(&self.block_num.to_be_bytes());

        /* data */
        bytes.extend_from_slice(self.data.as_slice());        

        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ParseError> {
        if bytes.len() < 5 { /* bounds check */
            return Err(ParseError::TooShort);
        }

        if bytes.len() > 516 { /* bounds check */
            return Err(ParseError::TooLong);
        }

        /* parse opcode */
        let opcode: MessageOpcode = ((bytes[0] as u16) << 8) |
                                            bytes[1] as u16; 

        /* this field is implicit in all message types, but we still need to
            validate the correctness of it in the wire format */
        let msg_type: Option<MessageType> = MessageType::from_opcode(opcode);

        if msg_type.is_none() { /* check for failure of our helper */
            return Err(ParseError::InvalidOpcode);
        }

        /* check the opcode actually matches the message type */
        if msg_type.unwrap() != MessageType::Data {
            return Err(ParseError::InvalidOpcode);
        }

        /* parse block number */
        let block_num: DataMessageBlockNumber = ((bytes[2] as u16) << 8) |
                                                    bytes[3] as u16;

        /* parse data */
        let data: Vec<u8> = bytes[4..].to_vec();
        
        /* actually construct message object */
        let message: DataMessage = DataMessage::new(block_num, data);

        Ok(message)
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

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        /* opcode */
        bytes.extend_from_slice(
            &MessageType::to_opcode(self.msg_type).to_be_bytes());

        /* block number */
        bytes.extend_from_slice(&self.block_num.to_be_bytes());
        
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ParseError> {
        if bytes.len() < 4 { /* bounds check */
            return Err(ParseError::TooShort);
        }

        if bytes.len() > 4 { /* bounds check */
            return Err(ParseError::TooLong);
        }

        /* parse opcode */
        let opcode: MessageOpcode = ((bytes[0] as u16) << 8) |
                                            bytes[1] as u16; 

        /* this field is implicit in all message types, but we still need to
            validate the correctness of it in the wire format */
        let msg_type: Option<MessageType> = MessageType::from_opcode(opcode);

        if msg_type.is_none() { /* check for failure of our helper */
            return Err(ParseError::InvalidOpcode);
        }

        /* check the opcode actually matches the message type */
        if msg_type.unwrap() != MessageType::Acknowledgement {
            return Err(ParseError::InvalidOpcode);
        }
       
        let block_num: DataMessageBlockNumber = ((bytes[2] as u16) << 8) |
                                                    bytes[3] as u16;

        let message: AcknowledgementMessage =
            AcknowledgementMessage::new(block_num);
    
        Ok(message)
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

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        /* opcode */
        bytes.extend_from_slice(
            &MessageType::to_opcode(self.msg_type).to_be_bytes());
        
        /* error code */
        bytes.extend_from_slice(&self.code.to_be_bytes());

        /* error message string */
        for byte in self.message.bytes() {
            bytes.push(byte);
        }

        bytes.push('\0' as u8); /* null terminate */

        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, ParseError> {
        if bytes.len() < 5 { /* bounds check */
            return Err(ParseError::TooShort);
        }

        /* parse opcode */
        let opcode: MessageOpcode = ((bytes[0] as u16) << 8) |
                                            bytes[1] as u16; 

        /* this field is implicit in all message types, but we still need to
            validate the correctness of it in the wire format */
        let msg_type: Option<MessageType> = MessageType::from_opcode(opcode);

        if msg_type.is_none() { /* check for failure of our helper */
            return Err(ParseError::InvalidOpcode);
        }

        /* check the opcode actually matches the message type */
        if msg_type.unwrap() != MessageType::Error {
            return Err(ParseError::InvalidOpcode);
        }
        
        /* parse error code */
        let error_code: ErrorMessageCode = ((bytes[2] as u16) << 8) |
                                                bytes[3] as u16;

        /* parse error message */
        let mut error_msg: String = String::new();
        let mut c: usize = 3;
        let mut curr_char: char = bytes[c] as char;

        if c + 2 >= bytes.len() { /* bounds check */
            return Err(ParseError::NoErrorMessage);
        }

        /* iterate over bytes, grabbing characters until null byte (we can do
            this because of the encoding of TFTP strings) */
        while curr_char != '\0' {
            if c >= bytes.len() - 1 { /* bounds check */
                return Err(ParseError::InvalidErrorMessage);
            }
            
            c += 1;
            curr_char = bytes[c] as char;
            error_msg.push(curr_char);
        }

        error_msg.pop();

        if error_msg.len() == 0 { /* bounds check */
            return Err(ParseError::NoErrorMessage);
        }

        /* actually construct message object */
        let message: ErrorMessage = ErrorMessage::new(error_code, error_msg);

        Ok(message)
    }
}

