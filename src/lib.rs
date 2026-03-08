#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![no_std]

use core::{
    error::Error,
    fmt::Display,
    ops::Deref,
    str::{Chars, FromStr, Utf8Error},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Str<const SIZE: usize>([u8; SIZE], usize);

impl<const SIZE: usize> Default for Str<SIZE> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const SIZE: usize> Str<SIZE> {
    /// Our bread and butter: &str to interface with the world.
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.0.get_unchecked(..self.1)) }
    }

    /// Returns the length of the buffer which the string is stored in.
    pub const fn buffer_size(&self) -> usize {
        SIZE
    }

    /// Returns the length of the underlying string slice.
    pub const fn len(&self) -> usize {
        self.1
    }

    pub fn is_empty(&self) -> bool {
       *self == Str::<SIZE>::default()
    }

    /// Creates an iterator over the underlying buffer's string slice
    pub fn chars(&self) -> Chars<'_> {
        self.as_str().chars()
    }

    /// Creates a new, stack allocated Str from a string slice.
    /// Panics if the size of the string is smaller than the buffer.
    /// If you wish to handle the error, use FromStr's implementation instead.
    pub fn new(val: &str) -> Str<SIZE> {
        assert!(val.len() <= SIZE);
        let mut buf = [0u8; SIZE];
        let inner = unsafe { buf.get_unchecked_mut(..val.len()) };
        inner.copy_from_slice(val.as_bytes());
        Self(buf, val.len())
    }

    /// Creates a new, stack allocated Str from a string slice.
    /// Panics if the size of the string is smaller than the buffer.
    /// # Safety
    ///
    /// Undefined behavior if the length of val is greater than the size  
    /// of the Str buffer.
    pub unsafe fn new_unchecked(val: &str) -> Str<SIZE> {
        let mut buf = [0u8; SIZE];
        let inner = unsafe { buf.get_unchecked_mut(..val.len()) };
        inner.copy_from_slice(val.as_bytes());
        Self(buf, val.len())
    }

    /// Attempts to create a new Str from a slice of bytes.
    /// # Safety
    ///
    /// Undefined Behavior if the bytes are not valid UTF-8.
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Str<SIZE> {
        let mut buf = [0u8; SIZE];
        let inner = unsafe { buf.get_unchecked_mut(..bytes.len()) };
        inner.copy_from_slice(bytes);
        Str(buf, bytes.len())
    }

    /// Attempts to create a new Str from a slice of bytes.
    /// Error conditions:
    ///     1) Byte slice is smaller than the buffer size.
    ///     2) Fails UTF-8 validation.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Str<SIZE>, StrErr> {
        if bytes.len() <= SIZE {
            Err(StrErr::MismatchedLength(MismatchedLengthDetails {
                bytes_read_size: bytes.len(),
                expected_size: SIZE,
            }))?
        }
        str::from_utf8(bytes).map_err(StrErr::Utf8Error)?;
        let mut buf = [0u8; SIZE];
        let inner = unsafe { buf.get_unchecked_mut(..bytes.len()) };
        inner.copy_from_slice(bytes);
        Ok(Str(buf, bytes.len()))
    }

    pub const fn empty() -> Self {
        Self([0u8; SIZE], 0)
    }

    /// Overwrites data in the existing buffer, regardless of the string slice size.
    /// Returns the old str from the allocation.
    pub fn try_overwrite(&mut self, str: &str) -> Result<Str<SIZE>, StrErr> {
        if str.len() <= SIZE {
            Err(StrErr::InsufficientSpace)?
        }
        let mut buf = [0u8; SIZE];
        let inner = unsafe { buf.get_unchecked_mut(..str.len()) };
        inner.copy_from_slice(str.as_bytes());
        core::mem::swap(&mut buf, &mut self.0);
        self.1 = str.len();
        Ok(Str(buf, buf.len()))
    }

    /// Overwrites data in the existing buffer, regardless of the string slice size.
    /// Returns the old str from the stack allocation.
    pub fn overwrite(&mut self, str: &str) -> Str<SIZE> {
        assert!(str.len() <= SIZE);
        let mut buf = [0u8; SIZE];
        let inner = unsafe { buf.get_unchecked_mut(..str.len()) };
        inner.copy_from_slice(str.as_bytes());
        core::mem::swap(&mut buf, &mut self.0);
        self.1 = str.len();
        Str(buf, buf.len())
    }

    /// Takes an existing Str from a stack allocation, leaving Str::<SIZE>::default() behind.
    pub fn take(&mut self) -> Str<SIZE> {
        let mut default = Str::<SIZE>::default();
        core::mem::swap(&mut default, self);
        default
    }

    /// Writes to a Str buffer. Panics if the string slice is not equal in size to the buffer.
    pub const fn write_exact(&mut self, str: &str) {
        self.0.copy_from_slice(str.as_bytes());
    }

    /// Writes Str to buffer. Guaranteed to not panic because type safety.
    /// WARNING: you might overwrite only part of existing data if the new data's string slice is
    /// smaller than the string slice in the other buffer.
    /// Example: overwriting b"larger_amount_of_data" with b"smol_data" would result in b"smol_dataount_of_data"
    pub fn write(&mut self, str: &Str<SIZE>) {
        self.1 = str.len();
        self.0.copy_from_slice(str.as_bytes());
    }

    /// Attempt to append the contents of a &str to an existing Str buffer.
    /// Errors out if there is insufficient room in the buffer (instead of truncating).
    pub fn try_append_str(&mut self, bytes: &str) -> Result<(), StrErr> {
        if SIZE - self.1 < bytes.len() {
            Err(StrErr::InsufficientSpace)?
        }
        let buf = unsafe { self.0.get_unchecked_mut(self.1..self.1 + bytes.len()) };
        buf.copy_from_slice(bytes.as_bytes());
        self.1 += bytes.len();
        Ok(())
    }

    /// Allocates a new Str buffer on the stack where the alloc size is equal to the sum of both Str stack alloc sizes.
    /// This function ignores unused bytes in the buffer.
    pub fn concat_str<const OTHER_SIZE: usize>(
        &self,
        other: &Str<OTHER_SIZE>,
    ) -> Str<{ SIZE + OTHER_SIZE }> {
        let mut buf: [u8; SIZE + OTHER_SIZE] = [0u8; SIZE + OTHER_SIZE];
        let (left, right) = unsafe { buf.split_at_mut_unchecked(self.1) };
        let right = unsafe { right.get_unchecked_mut(..other.1) };
        let s1 = unsafe { self.0.get_unchecked(0..self.1) };
        let s2 = unsafe { other.0.get_unchecked(0..other.1) };
        left.copy_from_slice(s1);
        right.copy_from_slice(s2);
        Str(buf, self.1 + other.1)
    }
}

impl<const SIZE: usize> AsMut<str> for Str<SIZE> {
    fn as_mut(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(self.0.get_unchecked_mut(..self.1)) }
    }
}

#[cfg(feature = "serde")]
pub mod serde_compatibility {
    use crate::{MismatchedLengthDetails, Str};
    use serde::{
        Deserialize, Serialize,
        de::{Expected, Visitor},
    };
    impl<const SIZE: usize> Serialize for Str<SIZE> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            unsafe { Ok(serializer.serialize_str(str::from_utf8_unchecked(&self.0))?) }
        }
    }

    impl<'de, const SIZE: usize> Deserialize<'de> for Str<SIZE> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(StrVisitor)
        }
    }

    pub struct StrVisitor<const SIZE: usize>;

    impl<'de, const SIZE: usize> Visitor<'_> for StrVisitor<SIZE> {
        type Value = Str<SIZE>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(formatter, "A string of length less than or equal to {SIZE}")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(serde::de::Error::custom)
        }

        fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
            Str::<SIZE>::try_from_bytes(&value).map_err(serde::de::Error::custom)
        }
    }

    impl Expected for MismatchedLengthDetails {
        fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(
                formatter,
                "String is guaranteed to be {} bytes, read {} bytes.",
                self.expected_size, self.bytes_read_size
            )
        }
    }
}
impl<const SIZE: usize> FromStr for Str<SIZE> {
    type Err = StrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() <= SIZE {
            Err(StrErr::MismatchedLength(MismatchedLengthDetails {
                bytes_read_size: s.len(),
                expected_size: SIZE,
            }))?
        }
        let mut buf = [0u8; SIZE];
        let inner = &mut buf[..s.len()];
        inner.copy_from_slice(s.as_bytes());
        Ok(Self(buf, s.len()))
    }
}

impl From<Utf8Error> for StrErr {
    fn from(value: Utf8Error) -> Self {
        StrErr::Utf8Error(value)
    }
}

#[derive(Debug)]
pub enum StrErr {
    MismatchedLength(MismatchedLengthDetails),
    Utf8Error(Utf8Error),
    InsufficientSpace,
}

#[derive(Debug)]
pub struct MismatchedLengthDetails {
    bytes_read_size: usize,
    expected_size: usize,
}

impl Display for StrErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StrErr::MismatchedLength(MismatchedLengthDetails {
                bytes_read_size,
                expected_size,
            }) => write!(
                f,
                "String is guaranteed to be {expected_size} bytes, read {bytes_read_size} bytes."
            ),
            StrErr::Utf8Error(utf8_error) => write!(f, "{utf8_error}"),
            StrErr::InsufficientSpace => {
                write!(f, "Insufficient room to append bytes to fixed size string")
            }
        }
    }
}

impl Error for StrErr {}

impl<const SIZE: usize> Display for Str<SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const SIZE: usize> TryFrom<&str> for Str<SIZE> {
    type Error = StrErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Str<SIZE> as FromStr>::from_str(value)
    }
}

impl<const SIZE: usize> Deref for Str<SIZE> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const SIZE: usize> AsRef<str> for Str<SIZE> {
    fn as_ref(&self) -> &str {
        self
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn basic() {
        let str: Str<12> = Str::new("hello world!");
        let string = "hello world!";
        assert_eq!(&*str, string);
    }

    #[test]
    fn concat_str() {
        let str0: Str<8> = Str::new("hello");
        let str1: Str<16> = Str::new(" world!");
        let new = str0.concat_str(&str1);
        assert_eq!(*new, *"hello world!");
        assert_eq!(new.buffer_size(), 24);
    }

    #[test]
    fn overwrite() {
        let mut str: Str<4> = Str::new("top");
        let str1: Str<4> = Str::new("kek");
        str.overwrite(&str1);
        assert_eq!(str, str1);
    }

    #[test]
    fn try_append() {
        let mut str: Str<10> = Str::new("bottom");
        let str1: Str<4> = Str::new(" kek");
        str.try_append_str(&str1).expect("buffer too small");
        assert_eq!(str, Str::new("bottom kek"))
    }

    #[test]
    #[should_panic]
    fn append_too_large() {
        let mut str: Str<9> = Str::new("bottom");
        let str1: Str<4> = Str::new(" kek");
        str.try_append_str(&str1).expect("buffer too small");
    }

    #[test]
    #[should_panic]
    fn write_too_small() {
        let mut str: Str<3> = Str::new("try");
        let str1: Str<2> = Str::new("me");
        str.write_exact(&str1);
    }

    #[test]
    #[should_panic]
    fn write_too_big() {
        let mut str: Str<3> = Str::new("try");
        let str1: Str<4> = Str::new("me");
        str.write_exact(&str1);
    }
}
