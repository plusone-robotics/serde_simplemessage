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

use std;
use std::fmt::{self, Display};

use serde::{de, ser};

/// Result type for SimpleMessage serialization and deserialization.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when serializing or deserializing SimpleMessage data.
#[derive(Debug)]
pub enum Error {
    /// Generic variant that can be created by data structures through the `ser::Error` and `de::Error` traits.
    Message(String),
    /// End of input reached unexpectedly.
    Eof,
    /// Value did not correspond to a valid enum variant.
    InvalidEnumValue(u32),
    /// Value could not be parsed as a valid Rust char (unicode scalar value).
    InvalidCharValue(u32),
    /// Sequence length is unknown.
    UnknownSeqLength,
    /// Sequence length is too large.
    InvalidSeqLength,
    /// Encoding/decoding identifiers is not supported.
    UnexpectedIdentifier,
    /// The data type cannot be trivially encoded/decoded in SimpleMessage format.
    UnsupportedData(String),
    /// The enum data type cannot be trivially encoded/decoded in SimpleMessage format.
    UnsupportedEnum(String),
    /// Encoding or decoding the numeric value would overflow the target type.
    Overflow,
    /// Trailing data was found after deserialization.
    TrailingData {bytes_read: usize, total_bytes: usize},
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            Error::InvalidEnumValue(value) => write!(formatter, "invalid enum value '{:?}'", value),
            Error::InvalidCharValue(value) => write!(formatter, "invalid char value '{:?}'", value),
            Error::UnknownSeqLength => formatter.write_str("sequence length must be known ahead of time"),
            Error::InvalidSeqLength => formatter.write_str("exceeded maximum sequence length"),
            Error::UnexpectedIdentifier => formatter.write_str("deserialize identifier is not supported"),
            Error::UnsupportedData(data_type) => write!(formatter, "unsupported data type '{:?}'", data_type),
            Error::UnsupportedEnum(enum_type) => write!(formatter, "unsupported enum type '{:?}'", enum_type),
            Error::Overflow => formatter.write_str("encoding or decoding the numeric value would overflow the target type"),
            Error::TrailingData {bytes_read, total_bytes} => write!(formatter, "trailing data was found after deserialization (read {bytes_read} bytes out of {total_bytes})"),
        }
    }
}

impl std::error::Error for Error {}
