//  Copyright 2026 Plus One Robotics
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde::{Deserialize, Serialize};

/// Communication types
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum CommType {
    Invalid = 0,
    /// One-way communication: the sender publishes updates, and the receiver does not respond.
    Topic = 1,
    /// Initial request of two-way communication: the sender sends a request, and the receiver sends a reply.
    ServiceRequest = 2,
    /// Response to two-way communication: the sender sends a request, and the receiver sends a reply.
    ServiceReply = 3,
}

/// Reply types (typically only used for ServiceReply). In cases of success or failure, the return data should include
/// the relevant return info.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum ReplyType {
    Invalid = 0,
    Success = 1,
    Failure = 2,
}

/// Prefix for a SimpleMessage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prefix {
    /// The total length of the message including the header, but *excluding* the size of this field.
    pub length: i32,
}
impl Prefix {
    pub fn new(length: i32) -> Self {
        Self { length }
    }
}

/// Header for a SimpleMessage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    /// The message type, specifying how the payload should be decoded.
    pub msg_type: i32,
    /// The communication type, specifying whether the message is a request, reply, or one-way topic update.
    pub comm_type: CommType,
    /// The reply code indicating success or failure, typically only used for ServiceReply messages.
    pub reply_code: ReplyType,
}
impl Header {
    pub fn new(msg_type: i32, comm_type: CommType, reply_code: ReplyType) -> Self {
        Self {
            msg_type,
            comm_type,
            reply_code,
        }
    }
}

/// Combined prefix and header for a SimpleMessage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixAndHeader {
    pub prefix: Prefix,
    pub header: Header,
}
impl PrefixAndHeader {
    pub fn new(msg_type: i32, comm_type: CommType, reply_code: ReplyType) -> Self {
        Self {
            prefix: Prefix::new(0),
            header: Header::new(msg_type, comm_type, reply_code),
        }
    }
}

/// The size of the SimpleMessage prefix in bytes.
pub const PREFIX_SIZE: usize = 4;
/// The size of the SimpleMessage header in bytes.
pub const HEADER_SIZE: usize = 12;
/// The size of the SimpleMessage prefix + header in bytes.
pub const PFH_SIZE: usize = PREFIX_SIZE + HEADER_SIZE;
