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

//! A library for encoding and decoding SimpleMessage messages.
//!
//! SimpleMessage is a messaging protocol used by the ROS-Industrial project for communication between a robot
//! controller and a host computer. It is a lightweight protocol that is easy to implement and has low overhead. It is
//! designed to be used over TCP or UDP sockets. The standard is formally defined in the [ROS-Industrial repository
//! on GitHub](https://github.com/ros-industrial/rep/blob/master/rep-I0006.rst).
//!
//! This library provides a way to encode and decode SimpleMessage messages in Rust using the serde library.
//!
//! # Example
//!
//! ```
//! use simple_message::{*, de::from_bytes, ser::encode_to_bytes};
//! use serde::{Serialize, Deserialize};
//!
//! // Define the message payload structure
//! #[derive(Debug, Serialize, Deserialize)]
//! struct MyMessage {
//!     field1: i32,
//!     field2: f32,
//! }
//! // Choose an identifier for the message type that doesn't conflict with those defined in standard_types.
//! const MY_MSG_TYPE: i32 = 10000;
//!
//! // Create the message payload
//! let header = Header::new(
//!     MY_MSG_TYPE,
//!     CommType::ServiceRequest,
//!     ReplyType::Invalid,
//! );
//! let msg = MyMessage { field1: 42, field2: 3.14 };
//!
//! // Encode the message, including header
//! let encoded = encode_to_bytes::<BigEndian, _>(&header, &msg).unwrap();
//! // (use to_bytes to encode just the payload with no header)
//!
//! println!("Encoded message: {:?}", encoded);
//!
//! // Decode the message, starting with the header to identify the payload decoding
//! let decoded_prefix = from_bytes::<BigEndian, Prefix>(&encoded[..PREFIX_SIZE]).unwrap();
//! assert!(decoded_prefix.length == encoded.len() as i32 - PREFIX_SIZE as i32);
//! let decoded_header = from_bytes::<BigEndian, Header>(&encoded[PREFIX_SIZE..PFH_SIZE]).unwrap();
//! match decoded_header.msg_type {
//!     MY_MSG_TYPE => {
//!         let decoded_msg = from_bytes::<BigEndian, MyMessage>(&encoded[PFH_SIZE..]).unwrap();
//!         println!("Decoded message: {:?}", decoded_msg);
//!     }
//!     _ => {
//!         println!("Unknown message type: {}", decoded_header.msg_type);
//!     }
//! }
//! ```
//!
//! # Gotchas to be aware of
//!
//! ## Not all Rust types can be encoded in SimpleMessage.
//!
//! Only the following types are supported:
//!
//! * `bool`
//! * `i8`, `i16`, `i32`, `i64`
//! * `u8`, `u16`, `u32`, `u64`
//! * `f32`, `f64`
//! * `char` (32-bit unicode scalar value)
//! * `[T; N]` (fixed-size arrays of supported types)
//! * `Vec<T>` (variable-size arrays of supported types)
//! * structs (with supported field types)
//! * trivial enums (with indexes or integer discriminants)
//! * tuples (of supported types)
//!
//! Any other types marked as `#[derive(Serialize, Deserialize)]` will compile, but will result in a runtime error
//! during encoding or decoding.
//!
//! ## `shared_bool` isn't formally defined in the REP
//!
//! The [ROS Enhancement Proposal that defines SimpleMessage](https://github.com/ros-industrial/rep/blob/master/rep-I0006.rst)
//! does not formally define 8-bit boolean values or raw byte data. However, the C++ implementation in that same
//! repository does and they are commonly accepted as part of the standard, so this crate chooses to implement them as
//! well.
//!
//! ## *Almost* all integer types are encoded as signed 32-bit ints.
//!
//! If your structure contains any signed integer types or unsigned types 16 bits or larger, they will be encoded as
//! 32-bit signed integers. When encoding, if the value to be encoded exceeds the range of a 32-bit signed integer, a
//! runtime error will occur. When decoding, if the decoded value exceeds the range of the destination type, a runtime
//! error will occur.
//!
//! **There is one exception**: `u8` values are encoded as raw bytes. This allows the user to easily operate on
//! individual bytes or byte arrays to transmit data that isn't otherwise explicitly handled by the protocol.
//!
//! ## All floating-point types are encoded as 32-bit floats.
//!
//! If your structure contains `f64` values, they will be truncated to 32-bit floats. When encoding, if the value to be
//! encoded exceeds the range of a 32-bit float, a runtime error will occur.
//!
//! The [REP](https://github.com/ros-industrial/rep/blob/master/rep-I0006.rst) specifies that `shared_real` *can* be
//! a 64-bit float if both the sender and the receiver agree. However, this is not common and this crate chooses not to
//! support this for the time being. Support may be added later.
//!
//! ## The protocol supports both big-endian and little-endian.
//!
//! The endianness of the wire format is not explicitly prescribed by the SimpleMessage protocol. For this reason, you
//! must explicitly specify the endianness when encoding and decoding. The endianness specified must match the
//! endianness expected by the other end of the connection.
//!
//! ## Variable-sized arrays are implicitly length-prefixed.
//!
//! Variable-sized arrays are encoded with a 4-byte unsigned length prefix, followed by the elements of the array. This
//! length prefix is not part of the data structure itself, so when decoding, the length prefix is consumed and the
//! remaining bytes are decoded as the elements of the array.
//!
//! Since this is not explicitly called out in the protocol documentation, there is no standard convention for this;
//! the `simple_message` crate chooses to use this length field to specify the *number of elements* in the sequence, not
//! the *number of bytes* in the sequence. Other implementations will not be supported by this crate.
//!
//! In API docs, this should be explicitly described, with a field specifying the length of the sequence, followed
//! immediately by an array of elements with length specified by the length field. This length field should nonetheless
//! be omitted by the rust structure, since it is implicit when you use a `Vec` type.
//!
//! ## You must use repr to use enum variants with integer values other than the default.
//!
//! By default, enum variants are encoded as a `i32` using their corresponding *index* (i.e. position in the source).
//! If you need to use specific integer values for enum variants, you must use `#[repr(<integer type>)]` and
//! `#[derive(Serialize_repr, Deserialize_repr)]` in order for those values to be used for encoding/decoding. Note that
//! all enum integer types except u8 will still be encoded as an `i32` on the wire.
//!
//! # Overview of the SimpleMessage protocol
//!
//! SimpleMessage has 4 fundamental datatypes over the wire:
//!
//! * shared_bool (1 byte)
//! * shared_int (signed two's complement; 4 bytes; endianness is determined by individual API spec)
//! * shared_real (IEEE 754 single precision float; 4 bytes, endianness is determined by individual API spec)
//! * raw bytes (transmitted as-is)
//!
//! Implicitly, it is possible to create structs of fixed size with a known field order which can be serialized and
//! deserialized, and API specifications typically include nested structures.
//!
//! API docs typically assign names to fields in structs, but these are only for documentation purposes and are not
//! encoded in the message. The order of fields is significant, and the receiver must know how to interpret the message
//! based on the message type specified in the header.
//!
//! A common pattern for variable-length messages that carry zero or more copies of a structure is to have a field
//! specifying the length of the sequence, followed immediately by the sequence of elements.
//!
//! API docs also often specify enumerations of values that are encoded as 4-byte signed integers.
//!
//! SimpleMessage messages have a header that contains the length of the message, the message type, the communication
//! type, and the reply code. The message type is used to determine how to deserialize the payload. The communication
//! type is used to determine whether the message is a request, reply, or one-way topic update. The reply code is only
//! set for a reply to a request and indicates whether the service invocation was possible, *not* whether the action the
//! request initiated was successful or not. In case of failure to invoke the service, implementations shall set
//! reply_code to FAILURE and any payload must be considered invalid by clients (ie: may be uninitialised).
//!
//! The exact meaning of the message type is defined by the API specification for the device you are communicating with.
//! A common set of message types is defined in the standard_types module, but you will likely need to define your own.
//!
//! # SimpleMessage Encoding
//!
//! ## Prefix format
//!
//! All SimpleMessage messages start with a 4-byte length prefix. According to the specification, this is not considered
//! part of the message, but it describes the combined length of the header and the message payload to follow.
//!
//! Note that since all integers in SimpleMessage are encoded as a *signed* integer, the length field is signed. This
//! means it's possible to send a prefix specifying a negative length, which would obviously be invalid.
//!
//! | Field | Type | Size | Description |
//! | ----- | ---- | ---- | ----------- |
//! | length | shared_int | 4 | The total length of the message including the header *excluding* the size of the length field itself. |
//!
//! Note that in this library, the length field is included as part of the `PrefixAndHeader` struct for convenience.
//!
//! ## Header format
//!
//! The next 12 bytes contain the header, which instructs the receiver how to handle the payload that follows.
//!
//! | Field | Type | Size | Description |
//! | ----- | ---- | ---- | ----------- |
//! | msg_type | shared_int | 4 | The message type, specifying how the payload should be decoded. |
//! | comm_type | shared_int | 4 | The communication type, specifying whether the message is a request, reply, or one-way topic update. |
//! | reply_code | shared_int | 4 | The reply code indicating success or failure, typically only used for ServiceReply messages. |
//!
//! ## Payload format
//!
//! The payload format is identified by the message type specified in the header. The precise definition of the payload
//! is application specific. Some standard message types and payload definitions are provided in the `standard_types`
//! module.
//!
//! As an example, here is a definition of the `JointTrajPt` payload (identified by message type
//! `MSG_TYPE_JOINT_TRAJ_PT`):
//!
//! ### `JointData`
//!
//! | Field | Type | Size | Description |
//! | ----- | ---- | ---- | ----------- |
//! | joints | shared_real\[10\] | 40 | The joint positions (units are specific to the robot controller). |
//!
//! ### `JointTrajPt`
//!
//! | Field | Type | Size | Description |
//! | ----- | ---- | ---- | ----------- |
//! | sequence | shared_int | 4 | The sequence number of the trajectory point. |
//! | joints | JointData | 40 | The joint positions for the trajectory point. |
//! | velocity | shared_real | 4 | The velocity of the trajectory point, specified in a form specific to the robot controller. |
//! | duration | shared_real | 4 | The duration the trajectory point should take to execute (units may be specific to the robott controller). |

// Re-export these types so clients don't have to explicitly depend on the byteorder crate.
pub use byteorder::{BigEndian, LittleEndian, ByteOrder};

pub mod types;
pub use types::{ CommType, ReplyType, Prefix, Header, PrefixAndHeader, HEADER_SIZE, PFH_SIZE, PREFIX_SIZE };

pub mod de;
pub mod error;
pub mod ser;
pub mod standard_types;

#[cfg(test)]
mod tests;
