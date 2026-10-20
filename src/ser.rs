use std::mem::size_of;

use byteorder::WriteBytesExt;
use serde::{ser, Serialize};

use crate::error::{Error, Result};
use crate::{ByteOrder, Header, PFH_SIZE, PREFIX_SIZE};

/// Serializer for SimpleMessage data.
pub struct Serializer<B: ByteOrder> {
    /// PhantomData to tell the compiler that this struct uses the byte order B.
    byte_order: std::marker::PhantomData<B>,
    /// The output buffer.
    output: Vec<u8>,
}

/// Serialize a value into a vector of bytes using SimpleMessage encoding using the specified byte order.
pub fn to_bytes<B: ByteOrder, T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut serializer: Serializer<B> = Serializer {
        byte_order: std::marker::PhantomData,
        output: vec![],
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

/// Encode a complete SimpleMessage, including prefix and header, into a vector of bytes using the specified byte order.
pub fn encode_to_bytes<B: ByteOrder, T: Serialize>(header: &Header, value: &T) -> Result<Vec<u8>> {
    let mut serializer: Serializer<B> = Serializer {
        byte_order: std::marker::PhantomData,
        output: vec![],
    };
    // Reserve space for the header and the message to reduce allocations. If T contains dynamically allocated data like
    // vectors, the output will be resized later, but for most messages this will avoid additional allocations.
    serializer.output.reserve(PFH_SIZE + size_of::<T>());
    // The prefix (first four bytes) are the length field, which will be overwritten at the end
    serializer.output.append(&mut vec![0; PREFIX_SIZE]);
    // Append the header next
    header.serialize(&mut serializer)?;
    // followed by the payload
    value.serialize(&mut serializer)?;
    // Go back and fill in the message length prefix. The message length does not include the size of the message length
    // field itself, but it includes the length of the rest of the header, so subtract 4 from the total.
    let msg_length = serializer.output.len() as i32 - PREFIX_SIZE as i32;
    B::write_i32(&mut serializer.output[..PREFIX_SIZE], msg_length);
    Ok(serializer.output)
}

impl<B: ByteOrder> ser::Serializer for &mut Serializer<B> {
    /// The output type produced by this `Serializer` during successful serialization.
    type Ok = ();

    /// The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing compound data structures like sequences
    // and maps. In this case no additional state is required beyond what is already stored in the Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    /// Bools are encoded as a single byte where true is 0x01 and false is 0x00.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output.push(if v { 1 } else { 0 } as u8);
        Ok(())
    }

    /// i8 values are sign-extended to i32.
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.write_i32::<B>(v as i32).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// i16 values are sign-extended to i32.
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.output.write_i32::<B>(v as i32).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// i32s are encoded as i32 with no conversion.
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.output.write_i32::<B>(v).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// i64 values are truncated to i32. If this exceeds the range of i32, an error is returned.
    fn serialize_i64(self, v: i64) -> Result<()> {
        if v < i32::MIN as i64 || v > i32::MAX as i64 {
            return Err(Error::Overflow);
        }
        self.output.write_i32::<B>(v as i32).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// u8 values, unlike other integers, are treated as raw bytes and encoded as-is in one byte.
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.write_u8(v).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// u16 values are sign-extended to i32.
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.write_i32::<B>(v as i32).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// u32 values are truncated to i32. If this exceeds the range of i32, an error is returned.
    fn serialize_u32(self, v: u32) -> Result<()> {
        if v > i32::MAX as u32 {
            return Err(Error::Overflow);
        }
        self.output.write_i32::<B>(v as i32).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// u64 values are truncated to i32. If this exceeds the range of i32, an error is returned.
    fn serialize_u64(self, v: u64) -> Result<()> {
        if v > i32::MAX as u64 {
            return Err(Error::Overflow);
        }
        self.output.write_i32::<B>(v as i32).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// f32s are encoded as f32 with no conversion.
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.output.write_f32::<B>(v).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// f64 values are truncated to f32. This will result in a loss of precision. If the value exceeds the range of f32,
    /// an error is returned.
    fn serialize_f64(self, v: f64) -> Result<()> {
        if v > f32::MAX as f64 || v < f32::MIN as f64 {
            return Err(Error::Overflow);
        }
        self.output.write_f32::<B>(v as f32).map_err(|e| Error::Message(e.to_string()))?;
        Ok(())
    }

    /// Rust chars are 32 bits for unicode support. For 8-bit ASCII chars or raw bytes, use u8.
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_u32(v as u32)
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_str(self, _v: &str) -> Result<()> {
        Err(Error::UnsupportedData("str".into()))
    }

    /// Variable-length byte sequences are length-prefixed with a u32.
    /// In serde, fixed-size byte arrays are serialized as tuples where the size is known a-priori.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.serialize_u32(v.len() as u32)?;
        self.output.extend_from_slice(v);
        Ok(())
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_none(self) -> Result<()> {
        Err(Error::UnsupportedData("Option::None".into()))
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_some<T: ?Sized + Serialize>(self, _v: &T) -> Result<()> {
        Err(Error::UnsupportedData("Option::Some".into()))
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_unit(self) -> Result<()> {
        Err(Error::UnsupportedData("()".into()))
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::UnsupportedData("()".into()))
    }

    /// By default, unit variants are encoded as a u32 using their corresponding index. For enum variants integer
    /// representations that do not match their index (e.g. variants that start at a nonzero value, skip values, or have
    /// negative values), use serde_repr to serialize as the underlying integer value instead.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.serialize_u32(variant_index)
    }

    /// Newtype structs are encoded as their inner type.
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()> {
        Err(Error::UnsupportedData("newtype variant".into()))
    }

    /// Sequences are length-prefixed with a u32; sequence length must be known ahead of time.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if len.is_none() {
            return Err(Error::UnknownSeqLength);
        }
        let len = len.unwrap();
        if len > u32::MAX as usize {
            return Err(Error::InvalidSeqLength);
        }
        self.serialize_u32(len as u32)?;
        // Values in the sequence are encoded using SerializeSeq below.
        Ok(self)
    }

    /// Fixed lengths of tuples are presumed to be known a-priori by both ends so length is not encoded.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        // Values in the tuple are encoded using SerializeTuple below.
        Ok(self)
    }

    /// Tuple structs are treated the same as structs.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        // Values in the tuple are encoded using SerializeTupleStruct below.
        Ok(self)
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedData("tuple variant".into()))
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::UnsupportedData("map".into()))
    }

    /// Structs are presumed to have a fixed size and field order known a-priori by both ends so length and field
    /// names are not encoded.
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        // Values in the struct are encoded using SerializeStruct below.
        Ok(self)
    }

    /// Unsupported datatype for SimpleMessage.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedData("struct variant".into()))
    }
}

impl<B: ByteOrder> ser::SerializeSeq for &mut Serializer<B> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<B: ByteOrder> ser::SerializeTuple for &mut Serializer<B> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<B: ByteOrder> ser::SerializeTupleStruct for &mut Serializer<B> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Unsupported datatype for SimpleMessage.
impl<B: ByteOrder> ser::SerializeTupleVariant for &mut Serializer<B> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::UnsupportedData("tuple variant".into()))
    }

    fn end(self) -> Result<()> {
        Err(Error::UnsupportedData("tuple variant".into()))
    }
}

/// Unsupported datatype for SimpleMessage.
impl<B: ByteOrder> ser::SerializeMap for &mut Serializer<B> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _key: &T) -> Result<()> {
        Err(Error::UnsupportedData("map key".into()))
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::UnsupportedData("map value".into()))
    }

    fn end(self) -> Result<()> {
        Err(Error::UnsupportedData("map".into()))
    }
}

impl<B: ByteOrder> ser::SerializeStruct for &mut Serializer<B> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<B: ByteOrder> ser::SerializeStructVariant for &mut Serializer<B> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, _value: &T) -> Result<()> {
        Err(Error::UnsupportedData("struct variant".into()))
    }

    fn end(self) -> Result<()> {
        Err(Error::UnsupportedData("struct variant".into()))
    }
}
