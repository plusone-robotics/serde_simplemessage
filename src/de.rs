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

use byteorder::ReadBytesExt;
use serde::{de, Deserialize};
use std::io::{Cursor, Read};

use crate::error::{Error, Result};
use crate::ByteOrder;

/// Deserializer for SimpleMessage data.
pub struct Deserializer<B: ByteOrder> {
    /// PhantomData to tell the compiler that this struct uses the byte order B.
    byte_order: std::marker::PhantomData<B>,
    /// The input buffer.
    input: Cursor<Vec<u8>>,
}

/// Deserialize a value from a vector of bytes using SimpleMessage encoding using the specified byte order.
pub fn from_bytes<'a, B: ByteOrder, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T> {
    // TODO: This is a fairly naive implementation that does some unnecessary copying. It can be optimized later, but
    // optimal performance isn't a high priority for this library at the moment.
    let mut deserializer: Deserializer<B> = Deserializer {
        byte_order: std::marker::PhantomData,
        input: Cursor::new(data.to_vec()),
    };
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.position() as usize != deserializer.input.get_ref().len() {
        return Err(Error::TrailingData {bytes_read: deserializer.input.position() as usize, total_bytes: deserializer.input.get_ref().len()});
    }
    Ok(t)
}

impl<'de, B: ByteOrder> de::Deserializer<'de> for &mut Deserializer<B> {
    type Error = Error;

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_any<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("any".into()))
    }

    /// Bools are encoded as a single byte where 0 is false and nonzero is true.
    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.read_u8().map_err(|_| Error::Eof)? != 0;
        visitor.visit_bool(value)
    }

    /// All integer types except u8s are encoded as i32. If the value is outside the range of i8, an error is returned.
    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.read_i32::<B>().map_err(|_| Error::Eof)?;
        if value < i8::MIN as i32 || value > i8::MAX as i32 {
            return Err(Error::Overflow);
        }
        visitor.visit_i8(value as i8)
    }

    /// All integer types except u8s are encoded as i32. If the value is outside the range of i8, an error is returned.
    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.read_i32::<B>().map_err(|_| Error::Eof)?;
        if value < i16::MIN as i32 || value > i16::MAX as i32 {
            return Err(Error::Overflow);
        }
        visitor.visit_i16(value as i16)
    }

    /// i32s are encoded as i32 with no conversion.
    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(self.input.read_i32::<B>().map_err(|_| Error::Eof)?)
    }

    /// i64s are encoded as i32 and there is no loss when decoding.
    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.input.read_i32::<B>().map_err(|_| Error::Eof)? as i64)
    }

    /// u8 values, unlike other integers, are treated as raw bytes and encoded as-is in one byte.
    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.input.read_u8().map_err(|_| Error::Eof)?)
    }

    /// All integer types except u8s are encoded as i32. If the value is outside the range of u16, an error is returned.
    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.read_i32::<B>().map_err(|_| Error::Eof)?;
        if value < 0 || value > u16::MAX as i32 {
            return Err(Error::Overflow);
        }
        visitor.visit_u16(value as u16)
    }

    /// All integer types except u8s are encoded as i32. If the value is outside the range of u32, an error is returned.
    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.read_i32::<B>().map_err(|_| Error::Eof)?;
        if value < 0 {
            return Err(Error::Overflow);
        }
        visitor.visit_u32(value as u32)
    }

    /// All integer types except u8s are encoded as i32. If the value is outside the range of u64, an error is returned.
    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.read_i32::<B>().map_err(|_| Error::Eof)?;
        if value < 0 {
            return Err(Error::Overflow);
        }
        visitor.visit_u64(value as u64)
    }

    /// f32s are encoded as f32 with no conversion.
    fn deserialize_f32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(self.input.read_f32::<B>().map_err(|_| Error::Eof)?)
    }

    /// f64s are encoded as f32s. There may be rounding differences when converting from f32 to f64.
    fn deserialize_f64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.read_f32::<B>().map_err(|_| Error::Eof)?;
        visitor.visit_f64(value as f64)
    }

    /// Rust chars are 32 bits for unicode support. For 8-bit ASCII chars or raw bytes, use u8.
    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.input.read_u32::<B>().map_err(|_| Error::Eof)?;
        match char::from_u32(value) {
            Some(c) => visitor.visit_char(c),
            None => Err(Error::InvalidCharValue(value)),
        }
    }

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_str<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("str".into()))
    }

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_string<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("String".into()))
    }

    /// Variable-length byte sequences are length-prefixed with a u32.
    /// In serde, fixed-size byte arrays are deserialized as tuples where the size is known a-priori.
    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = self.input.read_u32::<B>().map_err(|_| Error::Eof)? as usize;
        let mut buf = vec![0u8; len];
        self.input
            .read_exact(&mut buf)
            .map_err(|_| Error::Eof)?;
        visitor.visit_bytes(&buf)
    }

    /// Variable-length byte sequences are length-prefixed with a u32.
    /// In serde, fixed-size byte arrays are deserialized as tuples where the size is known a-priori.
    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = self.input.read_u32::<B>().map_err(|_| Error::Eof)? as usize;
        let mut buf = vec![0u8; len];
        self.input
            .read_exact(&mut buf)
            .map_err(|_| Error::Eof)?;
        visitor.visit_byte_buf(buf)
    }

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_option<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("Option".into()))
    }

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_unit<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("()".into()))
    }

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_unit_struct<V: de::Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("()".into()))
    }

    /// Newtype structs are encoded as their inner type.
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    /// Sequences are length-prefixed with a u32.
    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = self.input.read_u32::<B>().map_err(|_| Error::Eof)? as usize;
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len,
        })
    }

    /// Fixed lengths of tuples are presumed to be known a-priori by both ends so length is not encoded.
    fn deserialize_tuple<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len,
        })
    }

    /// Tuple structs are treated the same as structs.
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len,
        })
    }

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_map<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("map".into()))
    }

    /// Structs are presumed to have a fixed size and field order known a-priori by both ends so length and field
    /// names are not encoded.
    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        // Structs in SimpleMessage have fixed field order and no field names encoded,
        // so we deserialize them as a sequence of values
        visitor.visit_seq(SeqAccess {
            deserializer: self,
            len: fields.len(),
        })
    }

    /// By default, unit variants are encoded as a u32 using their corresponding index. For enum variants integer
    /// representations that do not match their index (e.g. variants that start at a nonzero value, skip values, or have
    /// negative values), use serde_repr to deserialize as the underlying integer value instead.
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let variant_index = self.input.read_u32::<B>().map_err(|_| Error::Eof)?;
        visitor.visit_enum(EnumAccess {
            deserializer: self,
            variant_index,
        })
    }

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_identifier<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnexpectedIdentifier)
    }

    /// Unsupported datatype for SimpleMessage.
    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("any".into()))
    }
}

struct SeqAccess<'a, B: ByteOrder> {
    deserializer: &'a mut Deserializer<B>,
    len: usize,
}

impl<'de, 'a, B: ByteOrder> de::SeqAccess<'de> for SeqAccess<'a, B> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            seed.deserialize(&mut *self.deserializer).map(Some)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct EnumAccess<'a, B: ByteOrder> {
    #[allow(dead_code)]
    deserializer: &'a mut Deserializer<B>,
    variant_index: u32,
}

impl<'de, 'a, B: ByteOrder> de::EnumAccess<'de> for EnumAccess<'a, B> {
    type Error = Error;
    type Variant = UnitVariant;

    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let val = seed.deserialize(VariantIndexDeserializer(self.variant_index))?;
        Ok((val, UnitVariant))
    }
}

struct VariantIndexDeserializer(u32);

impl<'de> de::Deserializer<'de> for VariantIndexDeserializer {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.0)
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.0)
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("bool".into()))
    }

    fn deserialize_i8<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("i8".into()))
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("i16".into()))
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("i32".into()))
    }

    fn deserialize_i64<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("i64".into()))
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("u8".into()))
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("u16".into()))
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("u64".into()))
    }

    fn deserialize_f32<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("f32".into()))
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("f64".into()))
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("char".into()))
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("str".into()))
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("String".into()))
    }

    fn deserialize_bytes<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("byte array".into()))
    }

    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("byte array".into()))
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("Option".into()))
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("()".into()))
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("()".into()))
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("newtype struct".into()))
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("seq".into()))
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("tuple".into()))
    }

    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("tuple struct".into()))
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("map".into()))
    }

    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("struct".into()))
    }

    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("enum".into()))
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.0)
    }

    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnum("any".into()))
    }
}

struct UnitVariant;

impl<'de> de::VariantAccess<'de> for UnitVariant {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, _seed: T) -> Result<T::Value> {
        Err(Error::UnsupportedData("newtype variant".into()))
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("tuple variant".into()))
    }

    fn struct_variant<V: de::Visitor<'de>>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedData("struct variant".into()))
    }
}
