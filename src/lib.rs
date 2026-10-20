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

#![doc = include_str!("../README.md")]

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
