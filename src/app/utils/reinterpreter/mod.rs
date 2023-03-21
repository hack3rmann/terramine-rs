#![macro_use]

//!
//! Provides some `type-byte` and `byte-type` reinterpretations to common types
//!

use {
    std::{mem::transmute, convert::TryInto},
    thiserror::Error,
};

/// Composes input list of `IntoIterator`s with `Item = u8`
/// into one large iterator by sequetially calling `.chain()`,
/// producing new `Iterator` with `Item = u8`.
#[macro_export]
macro_rules! compose {
    () => {
        Vec::<u8>::new()
            .into_iter()
    };

    ($once:expr $(,)?) => {
        $once.into_iter()
    };

    ($first:expr, $($next:expr),+ $(,)?) => {
        $first
            .into_iter()
            $(
                .chain($next)
            )+
    };
}

#[macro_export]
macro_rules! read {
    ($bytes:expr, $(let $var_name:ident $(:$VarType:ty)?),* $(,)?) => {
        let mut reader = ByteReader::new($bytes);
        $(
            let $var_name $(:$VarType)? = reader.read()?;
        )*
    };
}

pub use crate::{compose, read};

pub unsafe trait Reinterpret:
    AsBytes +
    FromBytes +
    StaticSizeHint +
    DynamicSize
{ }

unsafe impl<T:
    AsBytes +
    FromBytes +
    StaticSizeHint +
    DynamicSize
> Reinterpret for T { }

pub unsafe trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

pub unsafe trait FromBytes: Sized {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError>;
}

pub unsafe trait StaticSize: Sized {
    fn static_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

pub unsafe trait StaticSizeHint {
    fn static_size_hint() -> Option<usize>;
}

unsafe impl<T: StaticSize> StaticSizeHint for T {
    fn static_size_hint() -> Option<usize> {
        Some(Self::static_size())
    }
}

pub unsafe trait DynamicSize {
    fn dynamic_size(&self) -> usize;
}

unsafe impl<T: StaticSize> DynamicSize for T {
    fn dynamic_size(&self) -> usize {
        Self::static_size()
    }
}



#[derive(Error, Debug)]
pub enum ReinterpretError {
    #[error("not enough bytes, index is {idx} but source length is {len}")]
    NotEnoughBytes {
        idx: String,
        len: usize,
    },

    #[error("failed to convert types: {0}")]
    Conversion(String),
}

fn get<Idx, Out>(source: &[u8], idx: Idx) -> Result<&Out, ReinterpretError>
where
    Idx: std::slice::SliceIndex<[u8], Output = Out> + std::fmt::Debug + Clone,
{
    Ok(source.get(idx.clone())
        .ok_or_else(||
            ReinterpretError::NotEnoughBytes { 
                idx: format!("{:?}", idx),
                len: source.len(),
            }
        )?
    )
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ByteReader<'s> {
    pub bytes: &'s [u8],
}

impl<'s> ByteReader<'s> {
    pub fn new(source: &'s [u8]) -> Self {
        Self { bytes: source }
    }
    
    pub fn read<T>(&mut self) -> Result<T, ReinterpretError>
    where
        T: FromBytes + DynamicSize,
    {
        let result = T::from_bytes(self.bytes)?;
        let idx = result.dynamic_size()..;
        self.bytes = self.bytes.get(idx.clone())
            .ok_or_else(|| ReinterpretError::NotEnoughBytes {
                idx: format!("{:?}", idx),
                len: self.bytes.len()
            })?;

        Ok(result)
    }
}



unsafe impl AsBytes for u8 {
    fn as_bytes(&self) -> Vec<u8> { vec![*self] }
}

unsafe impl FromBytes for u8 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(*get(source, 0)?)
    }
}

unsafe impl StaticSize for u8 { }



unsafe impl AsBytes for i8 {
    fn as_bytes(&self) -> Vec<u8> { unsafe { vec![transmute(*self)] } }
}

unsafe impl FromBytes for i8 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe { transmute(*get(source, 0)?) })
    }
}

unsafe impl StaticSize for i8 { }



unsafe impl AsBytes for u16 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 2] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for u16 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([*get(source, 0)?, *get(source, 1)?])
        })
    }
}

unsafe impl StaticSize for u16 { }



unsafe impl AsBytes for i16 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 2] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for i16 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([*get(source, 0)?, *get(source, 1)?])
        })
    }
}

unsafe impl StaticSize for i16 { }



unsafe impl AsBytes for u32 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 4] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for u32 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([*get(source, 0)?, *get(source, 1)?, *get(source, 2)?, *get(source, 3)?])
        })
    }
}

unsafe impl StaticSize for u32 { }



unsafe impl AsBytes for i32 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 4] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for i32 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([*get(source, 0)?, *get(source, 1)?, *get(source, 2)?, *get(source, 3)?])
        })
    }
}

unsafe impl StaticSize for i32 { }



unsafe impl AsBytes for u64 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 8] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for u64 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([
                *get(source, 0)?, *get(source, 1)?, *get(source, 2)?, *get(source, 3)?,
                *get(source, 4)?, *get(source, 5)?, *get(source, 6)?, *get(source, 7)?,
            ])
        })
    }
}

unsafe impl StaticSize for u64 { }



unsafe impl AsBytes for i64 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 8] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for i64 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([
                *get(source, 0)?, *get(source, 1)?, *get(source, 2)?, *get(source, 3)?,
                *get(source, 4)?, *get(source, 5)?, *get(source, 6)?, *get(source, 7)?,
            ])
        })
    }
}

unsafe impl StaticSize for i64 { }



unsafe impl AsBytes for u128 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 16] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for u128 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([
                *get(source, 0)?, *get(source, 1)?, *get(source, 2)?, *get(source, 3)?,
                *get(source, 4)?, *get(source, 5)?, *get(source, 6)?, *get(source, 7)?,
                *get(source, 8)?, *get(source, 9)?, *get(source, 10)?, *get(source, 11)?,
                *get(source, 12)?, *get(source, 13)?, *get(source, 14)?, *get(source, 15)?,
            ])
        })
    }
}

unsafe impl StaticSize for u128 { }



unsafe impl AsBytes for i128 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 16] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for i128 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([
                *get(source, 0)?, *get(source, 1)?, *get(source, 2)?, *get(source, 3)?,
                *get(source, 4)?, *get(source, 5)?, *get(source, 6)?, *get(source, 7)?,
                *get(source, 8)?, *get(source, 9)?, *get(source, 10)?, *get(source, 11)?,
                *get(source, 12)?, *get(source, 13)?, *get(source, 14)?, *get(source, 15)?,
            ])
        })
    }
}

unsafe impl StaticSize for i128 { }



unsafe impl AsBytes for f32 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 4] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for f32 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([*get(source, 0)?, *get(source, 1)?, *get(source, 2)?, *get(source, 3)?])
        })
    }
}

unsafe impl StaticSize for f32 { }



unsafe impl AsBytes for f64 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 8] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl FromBytes for f64 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        Ok(unsafe {
            transmute([
                *get(source, 0)?, *get(source, 1)?, *get(source, 2)?, *get(source, 3)?,
                *get(source, 4)?, *get(source, 5)?, *get(source, 6)?, *get(source, 7)?,
            ])
        })
    }
}

unsafe impl StaticSize for f64 { }



unsafe impl AsBytes for usize {
    fn as_bytes(&self) -> Vec<u8> {
        let filled = *self as u64;
        filled.as_bytes()
    }
}

unsafe impl FromBytes for usize {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let filled = u64::from_bytes(source)?;
        filled.try_into()
            .map_err(|_| ReinterpretError::Conversion(
                format!("conversion of too large u64 ({filled}) to usize")
            ))
    }
}

unsafe impl StaticSize for usize {
    fn static_size() -> usize {
        u64::static_size()
    }
}



unsafe impl AsBytes for isize {
    fn as_bytes(&self) -> Vec<u8> {
        let filled = *self as i64;
        filled.as_bytes()
    }
}

unsafe impl FromBytes for isize {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let filled = i64::from_bytes(source)?;
        filled.try_into()
            .map_err(|_| ReinterpretError::Conversion(
                format!("conversion of too large i64 ({filled}) to isize")
            ))
    }
}

unsafe impl StaticSize for isize {
    fn static_size() -> usize {
        i64::static_size()
    }
}



unsafe impl AsBytes for char {
    fn as_bytes(&self) -> Vec<u8> {
        u32::from(*self).as_bytes()
    }
}

unsafe impl FromBytes for char {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let source = u32::from_bytes(source)?;
        source.try_into()
            .map_err(|_| ReinterpretError::Conversion(
                format!("conversion of non-UTF-8 u32 ({source}) to char")
            ))
    }
}

unsafe impl StaticSize for char {
    fn static_size() -> usize {
        u32::static_size()
    }
}



unsafe impl AsBytes for bool {
    fn as_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

unsafe impl FromBytes for bool {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let mut reader = ByteReader::new(source);
        let byte: u8 = reader.read()?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ReinterpretError::Conversion(
                "conversion of >1 byte to bool".into()
            ))
        }
    }
}

unsafe impl StaticSize for bool {
    fn static_size() -> usize {
        u8::static_size()
    }
}



unsafe impl<T: AsBytes> AsBytes for Vec<T> {
    fn as_bytes(&self) -> Vec<u8> {
        compose! {
            self.len().as_bytes(),
            self.iter()
                .flat_map(AsBytes::as_bytes),
        }.collect()
    }
}

unsafe impl<T: FromBytes + DynamicSize> FromBytes for Vec<T> {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let mut reader = ByteReader::new(source);
        let len = reader.read()?;

        let mut result = Self::with_capacity(len);

        for _ in 0..len {
            result.push(reader.read()?)
        }

        Ok(result)
    }
}

unsafe impl<T: StaticSize> DynamicSize for Vec<T> {
    fn dynamic_size(&self) -> usize {
        usize::static_size() + self.len() * T::static_size()
    }
}



unsafe impl AsBytes for bit_vec::BitVec {
    fn as_bytes(&self) -> Vec<u8> {
        compose! {
            self.len().as_bytes(),
            self.to_bytes(),
        }.collect()
    }
}

unsafe impl FromBytes for bit_vec::BitVec {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let mut reader = ByteReader::new(source);
        let len = reader.read()?;

        let mut result = Self::from_bytes(reader.bytes);
        result.truncate(len);

        Ok(result)
    }
}

unsafe impl DynamicSize for bit_vec::BitVec {
    fn dynamic_size(&self) -> usize {
        self.storage().len() + usize::static_size()
    }
}



unsafe impl<K, V> AsBytes for std::collections::HashMap<K, V>
where
    K: AsBytes,
    V: AsBytes,
{
    fn as_bytes(&self) -> Vec<u8> {
        compose! {
            self.len().as_bytes(),
            self.iter()
                .flat_map(|(key, value)| compose! {
                    key.as_bytes(),
                    value.as_bytes(),
                })
        }.collect()
    }
}

unsafe impl<K, V> FromBytes for std::collections::HashMap<K, V>
where
    K: DynamicSize + FromBytes + Eq + std::hash::Hash,
    V: DynamicSize + FromBytes,
{
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let mut reader = ByteReader::new(source);
        let len = reader.read()?;

        let mut result = Self::with_capacity(len);

        for _ in 0..len {
            result.insert(
                reader.read()?,
                reader.read()?,
            );
        }

        Ok(result)
    }
}

unsafe impl<K: StaticSize, V: StaticSize> DynamicSize for std::collections::HashMap<K, V> {
    fn dynamic_size(&self) -> usize {
        usize::static_size() + self.len() * (K::static_size() + V::static_size())
    }
}



unsafe impl<T: AsBytes> AsBytes for Option<T> {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            None => false.as_bytes(),

            Some(inner) => compose! {
                true.as_bytes(),
                inner.as_bytes(),
            }.collect(),
        }
    }
}

unsafe impl<T: FromBytes + DynamicSize> FromBytes for Option<T> {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let mut reader = ByteReader::new(source);
        let is_some: bool = reader.read()?;

        match is_some {
            false => Ok(None),
            true  => Ok(Some(reader.read()?))
        }
    }
}

unsafe impl<T: DynamicSize> DynamicSize for Option<T> {
    fn dynamic_size(&self) -> usize {
        bool::static_size() + 
        match self {
            None => 0,
            Some(inner) => inner.dynamic_size(),
        }
    }
}



use math_linear::prelude::*;

macro_rules! reinterpret_3d_vectors {
    ($($VecName:ident = ($x:ident, $y:ident, $z:ident): $Type:ty);* $(;)?) => {$(
        unsafe impl AsBytes for $VecName {
            fn as_bytes(&self) -> Vec<u8> {
                compose! {
                    self.$x.as_bytes(),
                    self.$y.as_bytes(),
                    self.$z.as_bytes(),
                }.collect()
            }
        }

        unsafe impl FromBytes for $VecName {
            fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
                let mut reader = ByteReader::new(source);

                Ok(Self::new(
                    reader.read()?,
                    reader.read()?,
                    reader.read()?,
                ))
            }
        }

        unsafe impl StaticSize for $VecName {
            fn static_size() -> usize { 3 * <$Type>::static_size() }
        }
    )*};
}

reinterpret_3d_vectors! {
    Byte3   = (x, y, z): i8;
    UByte3  = (x, y, z): u8;
    Short3  = (x, y, z): i16;
    UShort3 = (x, y, z): u16;
    Int3    = (x, y, z): i32;
    UInt3   = (x, y, z): u32;
    Long3   = (x, y, z): i64;
    ULong3  = (x, y, z): u64;
    Large3  = (x, y, z): i128;
    ULarge3 = (x, y, z): u128;
    ISize3  = (x, y, z): isize;
    USize3  = (x, y, z): usize;
    Float3  = (x, y, z): f32;
    Double3 = (x, y, z): f64;
    Color   = (r, g, b): f32;
    Color64 = (r, g, b): f64;
}



unsafe impl AsBytes for Float4 {
    fn as_bytes(&self) -> Vec<u8> {
        compose! {
            self.x().as_bytes(),
            self.y().as_bytes(),
            self.z().as_bytes(),
            self.w().as_bytes(),
        }.collect()
    }
}

unsafe impl FromBytes for Float4 {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let mut reader = ByteReader::new(source);

        let x: f32 = reader.read()?;
        let y: f32 = reader.read()?;
        let z: f32 = reader.read()?;
        let w: f32 = reader.read()?;

        Ok(Self::new(x, y, z, w))
    }
}

unsafe impl StaticSize for Float4 {
    fn static_size() -> usize { 16 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reinterpret_u8() {
        let before: u8 = 23;
        let after = u8::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(u8::static_size(), 1);
    }

    #[test]
    fn reinterpret_i8() {
        let before: i8 = 23;
        let after = i8::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(i8::static_size(), 1);
    }

    #[test]
    fn reinterpret_u16() {
        let before: u16 = 13243;
        let after = u16::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(u16::static_size(), 2);
    }

    #[test]
    fn reinterpret_i16() {
        let before: i16 = 1442;
        let after = i16::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(i16::static_size(), 2);
    }

    #[test]
    fn reinterpret_u32() {
        let before: u32 = 41432;
        let after = u32::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(u32::static_size(), 4);
    }

    #[test]
    fn reinterpret_i32() {
        let before: i32 = 2454;
        let after = i32::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(i32::static_size(), 4);
    }

    #[test]
    fn reinterpret_u64() {
        let before: u64 = 234;
        let after = u64::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(u64::static_size(), 8);
    }

    #[test]
    fn reinterpret_i64() {
        let before: i64 = 5424254;
        let after = i64::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(i64::static_size(), 8);
    }

    #[test]
    fn reinterpret_u128() {
        let before: u128 = 23452523453452334;
        let after = u128::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(u128::static_size(), 16);
    }

    #[test]
    fn reinterpret_i128() {
        let before: i128 = 243523452345;
        let after = i128::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(i128::static_size(), 16);
    }

    #[test]
    fn reinterpret_f32() {
        let before: f32 = 12.54;
        let after = f32::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(f32::static_size(), 4);
    }

    #[test]
    fn reinterpret_f64() {
        let before: f64 = 134442.4454;
        let after = f64::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(f64::static_size(), 8);
    }

    #[test]
    fn reinterpret_vec() {
        let before: Vec<i32> = vec![1, 124, 11, 44, 111, 4523, 765];
        let after = Vec::<i32>::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn reinterpret_bit_vec() {
        use bit_vec::BitVec;

        let mut before = BitVec::from_bytes(&[0b01001010, 0b00011000]);
        before.truncate(9);
        let after = <BitVec as FromBytes>::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn reinterpret_some() {
        let before: Option<i32> = Some(213);
        let after = Option::<i32>::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn reinterpret_none() {
        let before: Option<u128> = None;
        let after = Option::<u128>::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn reinterpret_usize() {
        let before: usize = 14242;
        let after = usize::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn reinterpret_isize() {
        let before: isize = 14242;
        let after = isize::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn reinterpret_vec_option() {
        let before: Vec<Option<i32>> = vec![Some(1), None, None, Some(12), None, Some(7327), Some(42)];
        let after: Vec<Option<i32>> = Vec::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn reinterpret_hash_map() {
        use std::collections::HashMap;

        let before = HashMap::from([
            ('a', vec![1, 2, 3, 4, 5]),
            ('b', vec![6, 7, 8, 9, 1]),
            ('c', vec![2, 4, 6, 8, 0]),
            ('d', vec![1, 3, 5, 7, 9]),
        ]);

        let after = HashMap::<char, Vec<i32>>::from_bytes(
            &before.as_bytes()
        ).unwrap();

        assert_eq!(before, after);
    }
}