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

use std::collections::HashMap as Map;

use crate::error::Result;
use crate::de::from_bytes;
use crate::ser::{encode_to_bytes, to_bytes};
use crate::{BigEndian, LittleEndian, CommType, ReplyType, Prefix, Header, PrefixAndHeader, PFH_SIZE};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

const TEST_MSG_TYPE: i32 = 63001;
const TEST_MSG_LENGTH: u8 = 84;

#[derive(Deserialize, Debug, Serialize)]
struct TestMessage {
    int_8: i8,
    int_16: i16,
    int_32: i32,
    int_64: i64,
    nested: Vec<TestNested>,
    bytes: [u8; 8],
    float_32: f32,
    float_64: f64,
}

/// Custom PartialEq implementation just for testing to account for floating point conversion errors
impl PartialEq for TestMessage {
    fn eq(&self, other: &Self) -> bool {
        const EPSILON: f64 = 0.0001;
        self.int_8 == other.int_8 &&
        self.int_16 == other.int_16 &&
        self.int_32 == other.int_32 &&
        self.int_64 == other.int_64 &&
        self.nested == other.nested &&
        self.bytes == other.bytes &&
        self.float_32 == other.float_32 && // f32 conversion should be lossless
        (self.float_64 - other.float_64).abs() < EPSILON // f64 conversion may lose precision
    }
}

#[derive(Deserialize_repr, Debug, PartialEq, Serialize_repr)]
#[repr(i32)]
enum TestEnum {
    Variant1 = 1000,
    Variant2 = -1001,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
struct TestNested {
    flag: bool,
    test_enum: TestEnum,
    uint_8: u8,
    uint_16: u16,
    uint_32: u32,
    uint_64: u64,
}

const TEST_HEADER: PrefixAndHeader = PrefixAndHeader {
    prefix: Prefix { length: TEST_MSG_LENGTH as i32 },
    header: Header {
        msg_type: TEST_MSG_TYPE,
        comm_type: CommType::Topic,
        reply_code: ReplyType::Invalid,
    },
};

const BE_BYTES: &[u8] = &[
    0, 0, 0, TEST_MSG_LENGTH,               // msg_length (excluding the length field itself)
    0, 0, 246, 25,                          // msg_type
    0, 0, 0, 1,                             // comm_type
    0, 0, 0, 0,                             // reply_code
    // TestMessage
    0, 0, 0, 1,                             // int_8
    255, 255, 255, 254,                     // int_16
    255, 255, 255, 253,                     // int_32
    0, 0, 0, 4,                             // int_64
    0, 0, 0, 2,                             // size of nested vector
    // first nested element
    1,                                      // flag
    0, 0, 3, 232,                           // TestEnum
    5,                                      // uint_8
    0, 0, 0, 6,                             // uint_16
    0, 0, 0, 7,                             // uint_32
    0, 0, 0, 8,                             // uint_64
    // second nested element
    0,                                      // flag
    255, 255, 252, 23,                      // TestEnum
    9,                                      // uint_8
    0, 0, 0, 10,                            // uint_16
    0, 0, 0, 11,                            // uint_32
    0, 0, 0, 12,                            // uint_64
    // back to TestMessage
    101, 102, 103, 104, 105, 106, 107, 108, // bytes
    65, 88, 0, 0,                           // float_32
    65, 97, 153, 154,                       // float_64
];

const LE_BYTES: &[u8] = &[
    TEST_MSG_LENGTH, 0, 0, 0,               // msg_length (excluding the length field itself)
    25, 246, 0, 0,                          // msg_type
    1, 0, 0, 0,                             // comm_type
    0, 0, 0, 0,                             // reply_code
    // TestMessage
    1, 0, 0, 0,                             // int_8
    254, 255, 255, 255,                     // int_16
    253, 255, 255, 255,                     // int_32
    4, 0, 0, 0,                             // int_64
    2, 0, 0, 0,                             // size of nested vector
    // first nested element
    1,                                      // flag
    232, 3, 0, 0,                           // TestEnum
    5,                                      // uint_8
    6, 0, 0, 0,                             // uint_16
    7, 0, 0, 0,                             // uint_32
    8, 0, 0, 0,                             // uint_64
    // second nested element
    0,                                      // flag
    23, 252, 255, 255,                      // TestEnum
    9,                                      // uint_8
    10, 0, 0, 0,                            // uint_16
    11, 0, 0, 0,                            // uint_32
    12, 0, 0, 0,                            // uint_64
    // back to TestMessage
    101, 102, 103, 104, 105, 106, 107, 108, // bytes
    00, 00, 88, 65,                         // float_32
    154, 153, 97, 65,                       // float_64
];

fn get_test_message() -> TestMessage {
    TestMessage {
        int_8: 1,
        int_16: -2,
        int_32: -3,
        int_64: 4,
        nested: vec![
            TestNested {
                flag: true,
                test_enum: TestEnum::Variant1,
                uint_8: 5,
                uint_16: 6,
                uint_32: 7,
                uint_64: 8,
            },
            TestNested {
                flag: false,
                test_enum: TestEnum::Variant2,
                uint_8: 9,
                uint_16: 10,
                uint_32: 11,
                uint_64: 12,
            },
        ],
        bytes: [101, 102, 103, 104, 105, 106, 107, 108],
        float_32: 13.5,
        float_64: 14.1,
    }
}

#[test]
fn test_to_bytes_big_endian() {
    let result: Vec<u8> = to_bytes::<BigEndian, _>(&get_test_message()).unwrap();
    assert_eq!(BE_BYTES[PFH_SIZE..], result);
}

#[test]
fn test_to_bytes_little_endian() {
    let result: Vec<u8> = to_bytes::<LittleEndian, _>(&get_test_message()).unwrap();
    assert_eq!(LE_BYTES[PFH_SIZE..], result);
}

#[test]
fn test_encode_to_bytes_big_endian() {
    let result: Vec<u8> = encode_to_bytes::<BigEndian, _>(
        &TEST_HEADER.header,
        &get_test_message()
    ).unwrap();
    assert_eq!(BE_BYTES, result);
}

#[test]
fn test_encode_to_bytes_little_endian() {
    let result: Vec<u8> = encode_to_bytes::<LittleEndian, _>(
        &TEST_HEADER.header,
        &get_test_message()
    ).unwrap();
    assert_eq!(LE_BYTES, result);
}

#[test]
fn test_from_bytes_big_endian() {
    let result: TestMessage = from_bytes::<BigEndian, _>(&BE_BYTES[PFH_SIZE..]).unwrap();
    assert_eq!(get_test_message(), result);
}

#[test]
fn test_from_bytes_little_endian() {
    let result: TestMessage = from_bytes::<LittleEndian, _>(&LE_BYTES[PFH_SIZE..]).unwrap();
    assert_eq!(get_test_message(), result);
}

#[test]
fn test_header_from_bytes_big_endian() {
    let result: PrefixAndHeader = from_bytes::<BigEndian, _>(&BE_BYTES[..PFH_SIZE]).unwrap();
    assert_eq!(TEST_HEADER, result);
}

#[test]
fn test_header_from_bytes_little_endian() {
    let result: PrefixAndHeader = from_bytes::<LittleEndian, _>(&LE_BYTES[..PFH_SIZE]).unwrap();
    assert_eq!(TEST_HEADER, result);
}

#[test]
fn test_roundtrip_big_endian() {
    let original = get_test_message();
    let encoded = encode_to_bytes::<BigEndian, _>(&TEST_HEADER.header, &original).unwrap();
    let decoded_header: PrefixAndHeader = from_bytes::<BigEndian, _>(&encoded[..PFH_SIZE]).unwrap();
    assert_eq!(TEST_HEADER, decoded_header);
    let decoded: TestMessage = from_bytes::<BigEndian, _>(&encoded[PFH_SIZE..]).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_roundtrip_little_endian() {
    let original = get_test_message();
    let encoded = encode_to_bytes::<LittleEndian, _>(&TEST_HEADER.header, &original).unwrap();
    let decoded_header: PrefixAndHeader = from_bytes::<LittleEndian, _>(&encoded[..PFH_SIZE]).unwrap();
    assert_eq!(TEST_HEADER, decoded_header);
    let decoded: TestMessage = from_bytes::<LittleEndian, _>(&encoded[PFH_SIZE..]).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_to_bytes_unsupported_types() {
    #[derive(Serialize)]
    enum UnsupportedEnum {
        // Variant holding a value is not supported by SimpleMessage
        Variant1(i32),
    }

    let result = to_bytes::<BigEndian, _>(&UnsupportedEnum::Variant1(42));
    assert!(result.is_err());

    // Strings are not supported by SimpleMessage
    let foo: String = "Hello World".into();
    let result = to_bytes::<BigEndian, _>(&foo);
    assert!(result.is_err());

    // Maps are not supported by SimpleMessage
    let bar: Map<String, i32> = [("a".into(), 1), ("b".into(), 2)].into_iter().collect();
    let result = to_bytes::<BigEndian, _>(&bar);
    assert!(result.is_err());
}

#[test]
fn test_from_bytes_invalid_enum() {
    let bytes = [0, 0, 0, 0]; // enum value 0 is not valid for TestEnum
    let result: Result<TestEnum> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_from_bytes_insufficient_data() {
    let result: Result<TestMessage> = from_bytes::<BigEndian, _>(&BE_BYTES[..1]);
    assert!(result.is_err());
}

#[test]
fn test_from_bytes_no_nested() {
    let bytes = [
        0, 0, 0, 1,                             // int_8
        255, 255, 255, 254,                     // int_16
        255, 255, 255, 253,                     // int_32
        0, 0, 0, 4,                             // int_64
        0, 0, 0, 0,                             // size of nested vector
        101, 102, 103, 104, 105, 106, 107, 108, // bytes
        65, 88, 0, 0,                           // float_32
        65, 97, 153, 154,                       // float_64
    ];
    let result: TestMessage = from_bytes::<BigEndian, _>(&bytes).unwrap();
    let expected: TestMessage = TestMessage {
        int_8: 1,
        int_16: -2,
        int_32: -3,
        int_64: 4,
        nested: vec![],
        bytes: [101, 102, 103, 104, 105, 106, 107, 108],
        float_32: 13.5,
        float_64: 14.1,
    };
    assert_eq!(result, expected);
}

#[test]
fn test_from_bytes_trailing_data() {
    let mut bytes = BE_BYTES[PFH_SIZE..].to_vec();
    bytes.push(42);
    let result: Result<TestMessage> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_to_bytes_overflow() {
    let value = i64::MAX;
    let result: Result<Vec<u8>> = to_bytes::<BigEndian, _>(&value);
    assert!(result.is_err());
    let value = i32::MAX;
    let result: Result<Vec<u8>> = to_bytes::<BigEndian, _>(&value);
    assert!(result.is_ok());
    let value = f64::MAX;
    let result: Result<Vec<u8>> = to_bytes::<BigEndian, _>(&value);
    assert!(result.is_err());
}

#[test]
fn test_from_bytes_overflow() {
    let bytes = i32::MAX.to_be_bytes();
    let result: Result<i8> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_err());
    let result: Result<i16> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_err());
    let result: Result<i32> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_ok());
    let result: Result<i64> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_ok());
    let result: Result<u8> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_err());
    let result: Result<u16> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_err());
    let bytes = i32::MIN.to_be_bytes();
    let result: Result<u32> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_err());
    let result: Result<u64> = from_bytes::<BigEndian, _>(&bytes);
    assert!(result.is_err());
}
