use byteorder::{ReadBytesExt, WriteBytesExt};
use paste::paste;
use std::io::{Read, Result as IoResult, Write};
use std::ops::{Deref, DerefMut};

pub trait Encode {
    fn encode(&self, writer: &mut impl Write) -> IoResult<()>;
}

pub trait Decode: Sized {
    fn decode(reader: &mut impl Read) -> IoResult<Self>;
}

// TODO: raw bool encoding is inefficient and should be replaced with bitsets.
impl Encode for bool {
    fn encode(&self, writer: &mut impl Write) -> IoResult<()> {
        writer.write_u8(if *self { 1 } else { 0 })
    }
}

impl Decode for bool {
    fn decode(reader: &mut impl Read) -> IoResult<Self> {
        Ok(reader.read_u8()? != 0)
    }
}

impl Encode for u8 {
    fn encode(&self, writer: &mut impl Write) -> IoResult<()> {
        writer.write_u8(*self)
    }
}

impl Decode for u8 {
    fn decode(reader: &mut impl Read) -> IoResult<Self> {
        reader.read_u8()
    }
}

impl Encode for i8 {
    fn encode(&self, writer: &mut impl Write) -> IoResult<()> {
        writer.write_i8(*self)
    }
}

impl Decode for i8 {
    fn decode(reader: &mut impl Read) -> IoResult<Self> {
        reader.read_i8()
    }
}

macro_rules! impl_ordered_int (
    ($type: ident) => (
        impl Encode for $type {
            fn encode(&self, writer: &mut impl Write) -> IoResult<()> {
                paste! { writer.[<write_ $type>]::<byteorder::LittleEndian>(*self) }
            }
        }

        impl Decode for $type {
            fn decode(reader: &mut impl Read) -> IoResult<Self> {
                paste! { reader.[<read_ $type>]::<byteorder::LittleEndian>() }
            }
        }
    )
);

impl_ordered_int!(u16);
impl_ordered_int!(u32);
impl_ordered_int!(u64);
impl_ordered_int!(u128);
impl_ordered_int!(i16);
impl_ordered_int!(i32);
impl_ordered_int!(i64);
impl_ordered_int!(i128);

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Var<T>(pub T);

impl<T> Deref for Var<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Var<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

macro_rules! impl_var_uint (
    ($type: ident) => (
        impl Encode for Var<$type> {
            fn encode(&self, writer: &mut impl Write) -> IoResult<()> {
                let mut value = self.0;

                loop {
                    if (value & !0x7f) == 0 {
                        writer.write(&[value as u8])?;
                        return Ok(());
                    }

                    let next = (value as u8 & 0x7f) | 0x80;
                    writer.write(&[next])?;
                    value >>= 7;
                }
            }
        }

        impl Decode for Var<$type> {
            fn decode(reader: &mut impl Read) -> IoResult<Self> {
                let mut value = 0;
                let mut position = 0;

                while position < $type::BITS {
                    let b = reader.read_u8()?;
                    value |= (b as $type & 0x7f) << position;

                    if (b & 0x80) == 0 {
                        break;
                    }

                    position += 7;
                }

                Ok(Var(value))
            }
        }
    )
);

impl_var_uint!(u16);
impl_var_uint!(u32);
impl_var_uint!(u64);
impl_var_uint!(u128);

impl Encode for String {
    fn encode(&self, writer: &mut impl Write) -> IoResult<()> {
        let len = Var::<u32>(self.len().try_into().unwrap());
        len.encode(writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl Decode for String {
    fn decode(reader: &mut impl Read) -> IoResult<Self> {
        let len = Var::<u32>::decode(reader)?;
        let mut buf = vec![0u8; len.0 as usize];
        reader.read_exact(&mut buf)?;

        if let Ok(string) = String::from_utf8(buf) {
            Ok(string)
        } else {
            Err(std::io::ErrorKind::InvalidData.into())
        }
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, writer: &mut impl Write) -> IoResult<()> {
        let len = Var::<u32>(self.len().try_into().unwrap());
        len.encode(writer)?;

        for item in self.iter() {
            item.encode(writer)?;
        }

        Ok(())
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(reader: &mut impl Read) -> IoResult<Self> {
        let len = Var::<u32>::decode(reader)?;
        let mut buf = Vec::with_capacity(len.0 as usize);
        for _ in 0..len.0 {
            buf.push(T::decode(reader)?);
        }

        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Debug;

    fn test_roundtrip<T: Debug + Eq + Encode + Decode>(original: T) {
        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();
        let mut reader = buf.as_slice();
        let decoded = T::decode(&mut reader).unwrap();
        assert_eq!(original, decoded, "Round-trip encoded values do not match!");
    }

    mod int {
        use super::*;

        macro_rules! test_int (
            ($type: ident) => (
                mod $type {
                    use super::*;

                    #[test]
                    fn min() {
                        test_roundtrip($type::MIN);
                    }

                    #[test]
                    fn max() {
                        test_roundtrip($type::MAX);
                    }

                    #[test]
                    fn one() {
                        test_roundtrip(1 as $type);
                    }
                }
            )
        );

        test_int!(u8);
        test_int!(u16);
        test_int!(u32);
        test_int!(u64);
        test_int!(u128);
        test_int!(i8);
        test_int!(i16);
        test_int!(i32);
        test_int!(i64);
        test_int!(i128);
    }

    mod var {
        use super::*;

        macro_rules! test_var_int (
            ($type: ident) => (
                mod $type {
                    use super::*;

                    #[test]
                    fn min() {
                        test_roundtrip(Var($type::MIN));
                    }

                    #[test]
                    fn max() {
                        test_roundtrip(Var($type::MAX));
                    }

                    #[test]
                    fn one() {
                        test_roundtrip(Var::<$type>(1));
                    }
                }
            )
        );

        test_var_int!(u16);
        test_var_int!(u32);
        test_var_int!(u64);
        test_var_int!(u128);
    }

    mod string {
        use super::*;

        #[test]
        fn hello_world() {
            test_roundtrip("Hello world!".to_string());
        }

        #[test]
        fn empty() {
            test_roundtrip("".to_string());
        }
    }
}
