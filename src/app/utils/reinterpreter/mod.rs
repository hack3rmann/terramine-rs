/**
 * Provides some `type-byte` and `byte-type` reinterpretations to common types
 */

use std::{mem::transmute, convert::TryInto};

pub unsafe trait Reinterpret:
    ReinterpretAsBytes +
    ReinterpretFromBytes +
    ReinterpretSize +
    StaticSizeHint +
    DynamicSize
{ }

unsafe impl<T:
    ReinterpretAsBytes +
    ReinterpretFromBytes +
    ReinterpretSize +
    StaticSizeHint +
    DynamicSize
> Reinterpret for T { }

pub unsafe trait ReinterpretAsBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

pub unsafe trait ReinterpretFromBytes: Sized {
    fn from_bytes(source: &[u8]) -> Option<Self>;
}

pub unsafe trait ReinterpretSize {
    fn reinterpret_size(&self) -> usize;
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



unsafe impl ReinterpretAsBytes for u8 {
    fn as_bytes(&self) -> Vec<u8> { vec![*self] }
}

unsafe impl ReinterpretFromBytes for u8 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(source[0])
    }
}

unsafe impl ReinterpretSize for u8 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u8 { }



unsafe impl ReinterpretAsBytes for i8 {
    fn as_bytes(&self) -> Vec<u8> { unsafe { vec![transmute(*self)] } }
}

unsafe impl ReinterpretFromBytes for i8 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe { transmute(source[0]) })
    }
}

unsafe impl ReinterpretSize for i8 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i8 { }



unsafe impl ReinterpretAsBytes for u16 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 2] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for u16 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1]])
        })
    }
}

unsafe impl ReinterpretSize for u16 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u16 { }



unsafe impl ReinterpretAsBytes for i16 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 2] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for i16 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1]])
        })
    }
}

unsafe impl ReinterpretSize for i16 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i16 { }



unsafe impl ReinterpretAsBytes for u32 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 4] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for u32 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1], source[2], source[3]])
        })
    }
}

unsafe impl ReinterpretSize for u32 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u32 { }



unsafe impl ReinterpretAsBytes for i32 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 4] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for i32 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1], source[2], source[3]])
        })
    }
}

unsafe impl ReinterpretSize for i32 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i32 { }



unsafe impl ReinterpretAsBytes for u64 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 8] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for u64 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7]])
        })
    }
}

unsafe impl ReinterpretSize for u64 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u64 { }



unsafe impl ReinterpretAsBytes for i64 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 8] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for i64 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7]])
        })
    }
}

unsafe impl ReinterpretSize for i64 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i64 { }



unsafe impl ReinterpretAsBytes for u128 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 16] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for u128 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1], source[2],  source[3],  source[4],  source[5],  source[6],  source[7],
                       source[8], source[9], source[10], source[11], source[12], source[13], source[14], source[15]])
        })
    }
}

unsafe impl ReinterpretSize for u128 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u128 { }



unsafe impl ReinterpretAsBytes for i128 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 16] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for i128 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1], source[2],  source[3],  source[4],  source[5],  source[6],  source[7],
                       source[8], source[9], source[10], source[11], source[12], source[13], source[14], source[15]])
        })
    }
}

unsafe impl ReinterpretSize for i128 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i128 { }



unsafe impl ReinterpretAsBytes for f32 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 4] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for f32 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1], source[2], source[3]])
        })
    }
}

unsafe impl ReinterpretSize for f32 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for f32 { }



unsafe impl ReinterpretAsBytes for f64 {
    fn as_bytes(&self) -> Vec<u8> {
        let bytes: [u8; 8] = unsafe { transmute(*self) };
        bytes.into()
    }
}

unsafe impl ReinterpretFromBytes for f64 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        Some(unsafe {
            transmute([source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7]])
        })
    }
}

unsafe impl ReinterpretSize for f64 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for f64 { }



unsafe impl ReinterpretAsBytes for usize {
    fn as_bytes(&self) -> Vec<u8> {
        let filled = *self as u64;
        filled.as_bytes()
    }
}

unsafe impl ReinterpretFromBytes for usize {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        let filled = u64::from_bytes(source)?;
        Some(filled as Self)
    }
}

unsafe impl ReinterpretSize for usize {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for usize {
    fn static_size() -> usize {
        u64::static_size()
    }
}



unsafe impl ReinterpretAsBytes for isize {
    fn as_bytes(&self) -> Vec<u8> {
        let filled = *self as i64;
        filled.as_bytes()
    }
}

unsafe impl ReinterpretFromBytes for isize {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        let filled = i64::from_bytes(source)?;
        Some(filled as Self)
    }
}

unsafe impl ReinterpretSize for isize {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for isize {
    fn static_size() -> usize {
        i64::static_size()
    }
}



unsafe impl ReinterpretAsBytes for char {
    fn as_bytes(&self) -> Vec<u8> {
        u32::from(*self).as_bytes()
    }
}

unsafe impl ReinterpretFromBytes for char {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        let source = u32::from_bytes(source)?;
        source.try_into().ok()
    }
}

unsafe impl ReinterpretSize for char {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for char {
    fn static_size() -> usize {
        u32::static_size()
    }
}



unsafe impl ReinterpretAsBytes for bool {
    fn as_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

unsafe impl ReinterpretFromBytes for bool {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        match source.get(0)? {
            &0 => Some(false),
            &1 => Some(true),
            _ => None,
        }
    }
}

unsafe impl ReinterpretSize for bool {
    fn reinterpret_size(&self) -> usize {
        Self::static_size()
    }
}

unsafe impl StaticSize for bool {
    fn static_size() -> usize {
        u8::static_size()
    }
}



unsafe impl<T: ReinterpretAsBytes> ReinterpretAsBytes for Vec<T> {
    fn as_bytes(&self) -> Vec<u8> {
        self.len()
            .as_bytes()
            .into_iter()
            .chain(self.iter()
                .flat_map(ReinterpretAsBytes::as_bytes)
            )
            .collect()
    }
}

unsafe impl<T: ReinterpretFromBytes + ReinterpretSize> ReinterpretFromBytes for Vec<T> {
    fn from_bytes(mut source: &[u8]) -> Option<Self> {
        let len = usize::from_bytes(source)?;
        source = &source[usize::static_size()..];

        let mut result = Vec::with_capacity(len);

        for _ in 0..len {
            let elem = T::from_bytes(source)?;
            source = &source[elem.reinterpret_size()..];

            result.push(elem);
        }

        Some(result)
    }
}

unsafe impl<T: ReinterpretSize> ReinterpretSize for Vec<T> {
    fn reinterpret_size(&self) -> usize {
        usize::static_size() +
        self.iter()
            .map(ReinterpretSize::reinterpret_size)
            .sum::<usize>()
    }
}

unsafe impl<T: StaticSize> DynamicSize for Vec<T> {
    fn dynamic_size(&self) -> usize {
        usize::static_size() + self.len() * T::static_size()
    }
}



unsafe impl<K, V> ReinterpretAsBytes for std::collections::HashMap<K, V>
where
    K: ReinterpretAsBytes,
    V: ReinterpretAsBytes,
{
    fn as_bytes(&self) -> Vec<u8> {
        self.len()
            .as_bytes()
            .into_iter()
            .chain(self.iter()
                .flat_map(|(key, value)| {
                    key.as_bytes()
                        .into_iter()
                        .chain(value.as_bytes())
                })
            )
            .collect()
    }
}

unsafe impl<K, V> ReinterpretFromBytes for std::collections::HashMap<K, V>
where
    K: ReinterpretSize + ReinterpretFromBytes + Eq + std::hash::Hash,
    V: ReinterpretSize + ReinterpretFromBytes,
{
    fn from_bytes(mut source: &[u8]) -> Option<Self> {
        let len = usize::from_bytes(source)?;
        source = &source[usize::static_size()..];

        let mut result = Self::with_capacity(len);

        while !source.is_empty() {
            let key = K::from_bytes(source)?;
            source = &source[key.reinterpret_size()..];

            let value = V::from_bytes(source)?;
            source = &source[value.reinterpret_size()..];

            result.insert(key, value);
        }

        Some(result)
    }
}

unsafe impl<K: ReinterpretSize, V: ReinterpretSize> ReinterpretSize for std::collections::HashMap<K, V> {
    fn reinterpret_size(&self) -> usize {
        usize::static_size() +
        self.iter()
            .map(|(key, value)|
                key.reinterpret_size() + value.reinterpret_size()
            )
            .sum::<usize>()
    }
}

unsafe impl<K: StaticSize, V: StaticSize> DynamicSize for std::collections::HashMap<K, V> {
    fn dynamic_size(&self) -> usize {
        usize::static_size() + self.len() * (K::static_size() + V::static_size())
    }
}



unsafe impl<T: ReinterpretAsBytes> ReinterpretAsBytes for Option<T> {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            None => false.as_bytes(),

            Some(inner) => true.as_bytes()
                .into_iter()
                .chain(inner.as_bytes())
                .collect(),
        }
    }
}

unsafe impl<T: ReinterpretFromBytes> ReinterpretFromBytes for Option<T> {
    fn from_bytes(mut source: &[u8]) -> Option<Self> {
        let is_some = bool::from_bytes(source)?;
        source = &source[bool::static_size()..];

        match is_some {
            false => Some(None),
            true  => Some(Some(T::from_bytes(source)?))
        }
    }
}

unsafe impl<T: ReinterpretSize> ReinterpretSize for Option<T> {
    fn reinterpret_size(&self) -> usize {
        match self {
            None => bool::static_size(),
            Some(inner) => bool::static_size() + inner.reinterpret_size(),
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
        unsafe impl ReinterpretAsBytes for $VecName {
            fn as_bytes(&self) -> Vec<u8> {
                let mut out = Vec::with_capacity(Self::static_size());

                out.append(&mut self.$x.as_bytes());
                out.append(&mut self.$y.as_bytes());
                out.append(&mut self.$z.as_bytes());

                return out;
            }
        }

        unsafe impl ReinterpretFromBytes for $VecName {
            fn from_bytes(source: &[u8]) -> Option<Self> {
                let size = <$Type>::static_size();

                let x = <$Type>::from_bytes(&source[0..size])?;
                let y = <$Type>::from_bytes(&source[size .. 2 * size])?;
                let z = <$Type>::from_bytes(&source[2 * size .. 3 * size])?;

                Some(Self::new(x, y, z))
            }
        }

        unsafe impl ReinterpretSize for $VecName {
            fn reinterpret_size(&self) -> usize { Self::static_size() }
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



unsafe impl ReinterpretAsBytes for Float4 {
    fn as_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::static_size());

        out.append(&mut self.x().as_bytes());
        out.append(&mut self.y().as_bytes());
        out.append(&mut self.z().as_bytes());
        out.append(&mut self.w().as_bytes());

        return out;
    }
}

unsafe impl ReinterpretFromBytes for Float4 {
    fn from_bytes(source: &[u8]) -> Option<Self> {
        let x = f32::from_bytes(&source[0..4])?;
        let y = f32::from_bytes(&source[4..8])?;
        let z = f32::from_bytes(&source[8..12])?;
        let w = f32::from_bytes(&source[12..16])?;

        Some(Self::new(x, y, z, w))
    }
}

unsafe impl ReinterpretSize for Float4 {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
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
        assert_eq!(before.reinterpret_size(), u8::static_size());
        assert_eq!(u8::static_size(), 1);
    }

    #[test]
    fn reinterpret_i8() {
        let before: i8 = 23;
        let after = i8::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), i8::static_size());
        assert_eq!(i8::static_size(), 1);
    }

    #[test]
    fn reinterpret_u16() {
        let before: u16 = 13243;
        let after = u16::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), u16::static_size());
        assert_eq!(u16::static_size(), 2);
    }

    #[test]
    fn reinterpret_i16() {
        let before: i16 = 1442;
        let after = i16::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), i16::static_size());
        assert_eq!(i16::static_size(), 2);
    }

    #[test]
    fn reinterpret_u32() {
        let before: u32 = 41432;
        let after = u32::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), u32::static_size());
        assert_eq!(u32::static_size(), 4);
    }

    #[test]
    fn reinterpret_i32() {
        let before: i32 = 2454;
        let after = i32::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), i32::static_size());
        assert_eq!(i32::static_size(), 4);
    }

    #[test]
    fn reinterpret_u64() {
        let before: u64 = 234;
        let after = u64::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), u64::static_size());
        assert_eq!(u64::static_size(), 8);
    }

    #[test]
    fn reinterpret_i64() {
        let before: i64 = 5424254;
        let after = i64::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), i64::static_size());
        assert_eq!(i64::static_size(), 8);
    }

    #[test]
    fn reinterpret_u128() {
        let before: u128 = 23452523453452334;
        let after = u128::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), u128::static_size());
        assert_eq!(u128::static_size(), 16);
    }

    #[test]
    fn reinterpret_i128() {
        let before: i128 = 243523452345;
        let after = i128::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), i128::static_size());
        assert_eq!(i128::static_size(), 16);
    }

    #[test]
    fn reinterpret_f32() {
        let before: f32 = 12.54;
        let after = f32::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), f32::static_size());
        assert_eq!(f32::static_size(), 4);
    }

    #[test]
    fn reinterpret_f64() {
        let before: f64 = 134442.4454;
        let after = f64::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), f64::static_size());
        assert_eq!(f64::static_size(), 8);
    }

    #[test]
    fn reinterpret_vec() {
        let before: Vec<i32> = vec![1, 124, 11, 44, 111, 4523, 765];
        let after = Vec::<i32>::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), usize::static_size() + before.len() * i32::static_size());
    }

    #[test]
    fn reinterpret_some() {
        let before: Option<i32> = Some(213);
        let after = Option::<i32>::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), i32::static_size() + 1);
    }

    #[test]
    fn reinterpret_none() {
        let before: Option<u128> = None;
        let after = Option::<u128>::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), 1);
    }

    #[test]
    fn reinterpret_usize() {
        let before: usize = 14242;
        let after = usize::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), usize::static_size());
        assert_eq!(before.reinterpret_size(), std::mem::size_of::<usize>());
    }

    #[test]
    fn reinterpret_isize() {
        let before: isize = 14242;
        let after = isize::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
        assert_eq!(before.reinterpret_size(), isize::static_size());
        assert_eq!(before.reinterpret_size(), std::mem::size_of::<isize>());
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