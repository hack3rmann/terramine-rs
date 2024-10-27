#![macro_use]

pub mod macros;
pub mod algorithm;

use directx_math::*;

make_2_component_vector! {
    #[derive(Hash, Eq)] pub Byte2   = (x, y): i8;
    #[derive(Hash, Eq)] pub UByte2  = (x, y): u8;
    #[derive(Hash, Eq)] pub Short2  = (x, y): i16;
    #[derive(Hash, Eq)] pub UShort2 = (x, y): u16;
    #[derive(Hash, Eq)] pub Int2    = (x, y): i32;
    #[derive(Hash, Eq)] pub UInt2   = (x, y): u32;
    #[derive(Hash, Eq)] pub Long2   = (x, y): i64;
    #[derive(Hash, Eq)] pub ULong2  = (x, y): u64;
    #[derive(Hash, Eq)] pub Large2  = (x, y): i128;
    #[derive(Hash, Eq)] pub ULarge2 = (x, y): u128;
    #[derive(Hash, Eq)] pub ISize2  = (x, y): isize;
    #[derive(Hash, Eq)] pub USize2  = (x, y): usize;
    pub Float2  = (x, y): f32;
    pub Double2 = (x, y): f64;
}

make_3_component_vector! {
    #[derive(Hash, Eq)] pub Byte3   = (x, y, z): i8;
    #[derive(Hash, Eq)] pub UByte3  = (x, y, z): u8;
    #[derive(Hash, Eq)] pub Short3  = (x, y, z): i16;
    #[derive(Hash, Eq)] pub UShort3 = (x, y, z): u16;
    #[derive(Hash, Eq)] pub Int3    = (x, y, z): i32;
    #[derive(Hash, Eq)] pub UInt3   = (x, y, z): u32;
    #[derive(Hash, Eq)] pub Long3   = (x, y, z): i64;
    #[derive(Hash, Eq)] pub ULong3  = (x, y, z): u64;
    #[derive(Hash, Eq)] pub Large3  = (x, y, z): i128;
    #[derive(Hash, Eq)] pub ULarge3 = (x, y, z): u128;
    #[derive(Hash, Eq)] pub ISize3  = (x, y, z): isize;
    #[derive(Hash, Eq)] pub USize3  = (x, y, z): usize;
    pub Float3  = (x, y, z): f32;
    pub Double3 = (x, y, z): f64;
    pub Color   = (r, g, b): f32;
    pub Color64 = (r, g, b): f64;
}

impl_neg! { Byte2   = (x, y) }
impl_neg! { Short2  = (x, y) }
impl_neg! { Int2    = (x, y) }
impl_neg! { Long2   = (x, y) }
impl_neg! { Large2  = (x, y) }
impl_neg! { ISize2  = (x, y) }
impl_neg! { Float2  = (x, y) }
impl_neg! { Double2 = (x, y) }
impl_neg! { Byte3   = (x, y, z) }
impl_neg! { Short3  = (x, y, z) }
impl_neg! { Int3    = (x, y, z) }
impl_neg! { Long3   = (x, y, z) }
impl_neg! { Large3  = (x, y, z) }
impl_neg! { ISize3  = (x, y, z) }
impl_neg! { Float3  = (x, y, z) }
impl_neg! { Double3 = (x, y, z) }

generate_3d_swizzles! {
    pub Byte3   -> Byte2   = (x, y, z): i8;
    pub UByte3  -> UByte2  = (x, y, z): u8;
    pub Short3  -> Short2  = (x, y, z): i16;
    pub UShort3 -> UShort2 = (x, y, z): u16;
    pub Int3    -> Int2    = (x, y, z): i32;
    pub UInt3   -> UInt2   = (x, y, z): u32;
    pub Long3   -> Long2   = (x, y, z): i64;
    pub ULong3  -> ULong2  = (x, y, z): u64;
    pub Large3  -> Large2  = (x, y, z): i128;
    pub ULarge3 -> ULarge2 = (x, y, z): u128;
    pub ISize3  -> ISize2  = (x, y, z): isize;
    pub USize3  -> USize2  = (x, y, z): usize;
    pub Float3  -> Float2  = (x, y, z): f32;
    pub Double3 -> Double2 = (x, y, z): f64;
}

generate_3d_swizzles_only_3d! {
    pub Color   = (r, g, b): f32;
    pub Color64 = (r, g, b): f32;
}

derive_froms_2_component! {
    Byte2 { x, y: i8 } <-> UByte2 { x, y: u8 };
    Byte2 { x, y: i8 } <-> Short2 { x, y: i16 };
    Byte2 { x, y: i8 } <-> UShort2 { x, y: u16 };
    Byte2 { x, y: i8 } <-> Int2 { x, y: i32 };
    Byte2 { x, y: i8 } <-> UInt2 { x, y: u32 };
    Byte2 { x, y: i8 } <-> Long2 { x, y: i64 };
    Byte2 { x, y: i8 } <-> ULong2 { x, y: u64 };
    Byte2 { x, y: i8 } <-> Large2 { x, y: i128 };
    Byte2 { x, y: i8 } <-> ULarge2 { x, y: u128 };
    Byte2 { x, y: i8 } <-> ISize2 { x, y: isize };
    Byte2 { x, y: i8 } <-> USize2 { x, y: usize };
    Byte2 { x, y: i8 } <-> Float2 { x, y: f32 };
    Byte2 { x, y: i8 } <-> Double2 { x, y: f64 };
    UByte2 { x, y: u8 } <-> Short2 { x, y: i16 };
    UByte2 { x, y: u8 } <-> UShort2 { x, y: u16 };
    UByte2 { x, y: u8 } <-> Int2 { x, y: i32 };
    UByte2 { x, y: u8 } <-> UInt2 { x, y: u32 };
    UByte2 { x, y: u8 } <-> Long2 { x, y: i64 };
    UByte2 { x, y: u8 } <-> ULong2 { x, y: u64 };
    UByte2 { x, y: u8 } <-> Large2 { x, y: i128 };
    UByte2 { x, y: u8 } <-> ULarge2 { x, y: u128 };
    UByte2 { x, y: u8 } <-> ISize2 { x, y: isize };
    UByte2 { x, y: u8 } <-> USize2 { x, y: usize };
    UByte2 { x, y: u8 } <-> Float2 { x, y: f32 };
    UByte2 { x, y: u8 } <-> Double2 { x, y: f64 };
    Short2 { x, y: i16 } <-> UShort2 { x, y: u16 };
    Short2 { x, y: i16 } <-> Int2 { x, y: i32 };
    Short2 { x, y: i16 } <-> UInt2 { x, y: u32 };
    Short2 { x, y: i16 } <-> Long2 { x, y: i64 };
    Short2 { x, y: i16 } <-> ULong2 { x, y: u64 };
    Short2 { x, y: i16 } <-> Large2 { x, y: i128 };
    Short2 { x, y: i16 } <-> ULarge2 { x, y: u128 };
    Short2 { x, y: i16 } <-> ISize2 { x, y: isize };
    Short2 { x, y: i16 } <-> USize2 { x, y: usize };
    Short2 { x, y: i16 } <-> Float2 { x, y: f32 };
    Short2 { x, y: i16 } <-> Double2 { x, y: f64 };
    UShort2 { x, y: u16 } <-> Int2 { x, y: i32 };
    UShort2 { x, y: u16 } <-> UInt2 { x, y: u32 };
    UShort2 { x, y: u16 } <-> Long2 { x, y: i64 };
    UShort2 { x, y: u16 } <-> ULong2 { x, y: u64 };
    UShort2 { x, y: u16 } <-> Large2 { x, y: i128 };
    UShort2 { x, y: u16 } <-> ULarge2 { x, y: u128 };
    UShort2 { x, y: u16 } <-> ISize2 { x, y: isize };
    UShort2 { x, y: u16 } <-> USize2 { x, y: usize };
    UShort2 { x, y: u16 } <-> Float2 { x, y: f32 };
    UShort2 { x, y: u16 } <-> Double2 { x, y: f64 };
    Int2 { x, y: i32 } <-> UInt2 { x, y: u32 };
    Int2 { x, y: i32 } <-> Long2 { x, y: i64 };
    Int2 { x, y: i32 } <-> ULong2 { x, y: u64 };
    Int2 { x, y: i32 } <-> Large2 { x, y: i128 };
    Int2 { x, y: i32 } <-> ULarge2 { x, y: u128 };
    Int2 { x, y: i32 } <-> ISize2 { x, y: isize };
    Int2 { x, y: i32 } <-> USize2 { x, y: usize };
    Int2 { x, y: i32 } <-> Float2 { x, y: f32 };
    Int2 { x, y: i32 } <-> Double2 { x, y: f64 };
    UInt2 { x, y: u32 } <-> Long2 { x, y: i64 };
    UInt2 { x, y: u32 } <-> ULong2 { x, y: u64 };
    UInt2 { x, y: u32 } <-> Large2 { x, y: i128 };
    UInt2 { x, y: u32 } <-> ULarge2 { x, y: u128 };
    UInt2 { x, y: u32 } <-> ISize2 { x, y: isize };
    UInt2 { x, y: u32 } <-> USize2 { x, y: usize };
    UInt2 { x, y: u32 } <-> Float2 { x, y: f32 };
    UInt2 { x, y: u32 } <-> Double2 { x, y: f64 };
    Long2 { x, y: i64 } <-> ULong2 { x, y: u64 };
    Long2 { x, y: i64 } <-> Large2 { x, y: i128 };
    Long2 { x, y: i64 } <-> ULarge2 { x, y: u128 };
    Long2 { x, y: i64 } <-> ISize2 { x, y: isize };
    Long2 { x, y: i64 } <-> USize2 { x, y: usize };
    Long2 { x, y: i64 } <-> Float2 { x, y: f32 };
    Long2 { x, y: i64 } <-> Double2 { x, y: f64 };
    ULong2 { x, y: u64 } <-> Large2 { x, y: i128 };
    ULong2 { x, y: u64 } <-> ULarge2 { x, y: u128 };
    ULong2 { x, y: u64 } <-> ISize2 { x, y: isize };
    ULong2 { x, y: u64 } <-> USize2 { x, y: usize };
    ULong2 { x, y: u64 } <-> Float2 { x, y: f32 };
    ULong2 { x, y: u64 } <-> Double2 { x, y: f64 };
    Large2 { x, y: i128 } <-> ULarge2 { x, y: u128 };
    Large2 { x, y: i128 } <-> ISize2 { x, y: isize };
    Large2 { x, y: i128 } <-> USize2 { x, y: usize };
    Large2 { x, y: i128 } <-> Float2 { x, y: f32 };
    Large2 { x, y: i128 } <-> Double2 { x, y: f64 };
    ULarge2 { x, y: u128 } <-> ISize2 { x, y: isize };
    ULarge2 { x, y: u128 } <-> USize2 { x, y: usize };
    ULarge2 { x, y: u128 } <-> Float2 { x, y: f32 };
    ULarge2 { x, y: u128 } <-> Double2 { x, y: f64 };
    ISize2 { x, y: isize } <-> USize2 { x, y: usize };
    ISize2 { x, y: isize } <-> Float2 { x, y: f32 };
    ISize2 { x, y: isize } <-> Double2 { x, y: f64 };
    USize2 { x, y: usize } <-> Float2 { x, y: f32 };
    USize2 { x, y: usize } <-> Double2 { x, y: f64 };
    Float2 { x, y: f32 } <-> Double2 { x, y: f64 };
}

derive_froms_3_component! {
    Byte3   { x, y, z: i8 }    <-> UByte3  { x, y, z: u8 };
    Byte3   { x, y, z: i8 }    <-> Short3  { x, y, z: i16 };
    Byte3   { x, y, z: i8 }    <-> UShort3 { x, y, z: u16 };
    Byte3   { x, y, z: i8 }    <-> Int3    { x, y, z: i32 };
    Byte3   { x, y, z: i8 }    <-> UInt3   { x, y, z: u32 };
    Byte3   { x, y, z: i8 }    <-> Long3   { x, y, z: i64 };
    Byte3   { x, y, z: i8 }    <-> ULong3  { x, y, z: u64 };
    Byte3   { x, y, z: i8 }    <-> Large3  { x, y, z: i128 };
    Byte3   { x, y, z: i8 }    <-> ULarge3 { x, y, z: u128 };
    Byte3   { x, y, z: i8 }    <-> ISize3  { x, y, z: isize };
    Byte3   { x, y, z: i8 }    <-> USize3  { x, y, z: usize };
    Byte3   { x, y, z: i8 }    <-> Float3  { x, y, z: f32 };
    Byte3   { x, y, z: i8 }    <-> Double3 { x, y, z: f64 };
    Byte3   { x, y, z: i8 }    <-> Color   { r, g, b: f32 };
    Byte3   { x, y, z: i8 }    <-> Color64 { r, g, b: f64 };
    UByte3  { x, y, z: u8 }    <-> Short3  { x, y, z: i16 };
    UByte3  { x, y, z: u8 }    <-> UShort3 { x, y, z: u16 };
    UByte3  { x, y, z: u8 }    <-> Int3    { x, y, z: i32 };
    UByte3  { x, y, z: u8 }    <-> UInt3   { x, y, z: u32 };
    UByte3  { x, y, z: u8 }    <-> Long3   { x, y, z: i64 };
    UByte3  { x, y, z: u8 }    <-> ULong3  { x, y, z: u64 };
    UByte3  { x, y, z: u8 }    <-> Large3  { x, y, z: i128 };
    UByte3  { x, y, z: u8 }    <-> ULarge3 { x, y, z: u128 };
    UByte3  { x, y, z: u8 }    <-> ISize3  { x, y, z: isize };
    UByte3  { x, y, z: u8 }    <-> USize3  { x, y, z: usize };
    UByte3  { x, y, z: u8 }    <-> Float3  { x, y, z: f32 };
    UByte3  { x, y, z: u8 }    <-> Double3 { x, y, z: f64 };
    UByte3  { x, y, z: u8 }    <-> Color   { r, g, b: f32 };
    UByte3  { x, y, z: u8 }    <-> Color64 { r, g, b: f64 };
    Short3  { x, y, z: i16 }   <-> UShort3 { x, y, z: u16 };
    Short3  { x, y, z: i16 }   <-> Int3    { x, y, z: i32 };
    Short3  { x, y, z: i16 }   <-> UInt3   { x, y, z: u32 };
    Short3  { x, y, z: i16 }   <-> Long3   { x, y, z: i64 };
    Short3  { x, y, z: i16 }   <-> ULong3  { x, y, z: u64 };
    Short3  { x, y, z: i16 }   <-> Large3  { x, y, z: i128 };
    Short3  { x, y, z: i16 }   <-> ULarge3 { x, y, z: u128 };
    Short3  { x, y, z: i16 }   <-> ISize3  { x, y, z: isize };
    Short3  { x, y, z: i16 }   <-> USize3  { x, y, z: usize };
    Short3  { x, y, z: i16 }   <-> Float3  { x, y, z: f32 };
    Short3  { x, y, z: i16 }   <-> Double3 { x, y, z: f64 };
    Short3  { x, y, z: i16 }   <-> Color   { r, g, b: f32 };
    Short3  { x, y, z: i16 }   <-> Color64 { r, g, b: f64 };
    UShort3 { x, y, z: u16 }   <-> Int3    { x, y, z: i32 };
    UShort3 { x, y, z: u16 }   <-> UInt3   { x, y, z: u32 };
    UShort3 { x, y, z: u16 }   <-> Long3   { x, y, z: i64 };
    UShort3 { x, y, z: u16 }   <-> ULong3  { x, y, z: u64 };
    UShort3 { x, y, z: u16 }   <-> Large3  { x, y, z: i128 };
    UShort3 { x, y, z: u16 }   <-> ULarge3 { x, y, z: u128 };
    UShort3 { x, y, z: u16 }   <-> ISize3  { x, y, z: isize };
    UShort3 { x, y, z: u16 }   <-> USize3  { x, y, z: usize };
    UShort3 { x, y, z: u16 }   <-> Float3  { x, y, z: f32 };
    UShort3 { x, y, z: u16 }   <-> Double3 { x, y, z: f64 };
    UShort3 { x, y, z: u16 }   <-> Color   { r, g, b: f32 };
    UShort3 { x, y, z: u16 }   <-> Color64 { r, g, b: f64 };
    Int3    { x, y, z: i32 }   <-> UInt3   { x, y, z: u32 };
    Int3    { x, y, z: i32 }   <-> Long3   { x, y, z: i64 };
    Int3    { x, y, z: i32 }   <-> ULong3  { x, y, z: u64 };
    Int3    { x, y, z: i32 }   <-> Large3  { x, y, z: i128 };
    Int3    { x, y, z: i32 }   <-> ULarge3 { x, y, z: u128 };
    Int3    { x, y, z: i32 }   <-> ISize3  { x, y, z: isize };
    Int3    { x, y, z: i32 }   <-> USize3  { x, y, z: usize };
    Int3    { x, y, z: i32 }   <-> Float3  { x, y, z: f32 };
    Int3    { x, y, z: i32 }   <-> Double3 { x, y, z: f64 };
    Int3    { x, y, z: i32 }   <-> Color   { r, g, b: f32 };
    Int3    { x, y, z: i32 }   <-> Color64 { r, g, b: f64 };
    UInt3   { x, y, z: u32 }   <-> Long3   { x, y, z: i64 };
    UInt3   { x, y, z: u32 }   <-> ULong3  { x, y, z: u64 };
    UInt3   { x, y, z: u32 }   <-> Large3  { x, y, z: i128 };
    UInt3   { x, y, z: u32 }   <-> ULarge3 { x, y, z: u128 };
    UInt3   { x, y, z: u32 }   <-> ISize3  { x, y, z: isize };
    UInt3   { x, y, z: u32 }   <-> USize3  { x, y, z: usize };
    UInt3   { x, y, z: u32 }   <-> Float3  { x, y, z: f32 };
    UInt3   { x, y, z: u32 }   <-> Double3 { x, y, z: f64 };
    UInt3   { x, y, z: u32 }   <-> Color   { r, g, b: f32 };
    UInt3   { x, y, z: u32 }   <-> Color64 { r, g, b: f64 };
    Long3   { x, y, z: i64 }   <-> ULong3  { x, y, z: u64 };
    Long3   { x, y, z: i64 }   <-> Large3  { x, y, z: i128 };
    Long3   { x, y, z: i64 }   <-> ULarge3 { x, y, z: u128 };
    Long3   { x, y, z: i64 }   <-> ISize3  { x, y, z: isize };
    Long3   { x, y, z: i64 }   <-> USize3  { x, y, z: usize };
    Long3   { x, y, z: i64 }   <-> Float3  { x, y, z: f32 };
    Long3   { x, y, z: i64 }   <-> Double3 { x, y, z: f64 };
    Long3   { x, y, z: i64 }   <-> Color   { r, g, b: f32 };
    Long3   { x, y, z: i64 }   <-> Color64 { r, g, b: f64 };
    ULong3  { x, y, z: u64 }   <-> Large3  { x, y, z: i128 };
    ULong3  { x, y, z: u64 }   <-> ULarge3 { x, y, z: u128 };
    ULong3  { x, y, z: u64 }   <-> ISize3  { x, y, z: isize };
    ULong3  { x, y, z: u64 }   <-> USize3  { x, y, z: usize };
    ULong3  { x, y, z: u64 }   <-> Float3  { x, y, z: f32 };
    ULong3  { x, y, z: u64 }   <-> Double3 { x, y, z: f64 };
    ULong3  { x, y, z: u64 }   <-> Color   { r, g, b: f32 };
    ULong3  { x, y, z: u64 }   <-> Color64 { r, g, b: f64 };
    Large3  { x, y, z: i128 }  <-> ULarge3 { x, y, z: u128 };
    Large3  { x, y, z: i128 }  <-> ISize3  { x, y, z: isize };
    Large3  { x, y, z: i128 }  <-> USize3  { x, y, z: usize };
    Large3  { x, y, z: i128 }  <-> Float3  { x, y, z: f32 };
    Large3  { x, y, z: i128 }  <-> Double3 { x, y, z: f64 };
    Large3  { x, y, z: i128 }  <-> Color   { r, g, b: f32 };
    Large3  { x, y, z: i128 }  <-> Color64 { r, g, b: f64 };
    ULarge3 { x, y, z: u128 }  <-> ISize3  { x, y, z: isize };
    ULarge3 { x, y, z: u128 }  <-> USize3  { x, y, z: usize };
    ULarge3 { x, y, z: u128 }  <-> Float3  { x, y, z: f32 };
    ULarge3 { x, y, z: u128 }  <-> Double3 { x, y, z: f64 };
    ULarge3 { x, y, z: u128 }  <-> Color   { r, g, b: f32 };
    ULarge3 { x, y, z: u128 }  <-> Color64 { r, g, b: f64 };
    ISize3  { x, y, z: isize } <-> USize3  { x, y, z: usize };
    ISize3  { x, y, z: isize } <-> Float3  { x, y, z: f32 };
    ISize3  { x, y, z: isize } <-> Double3 { x, y, z: f64 };
    ISize3  { x, y, z: isize } <-> Color   { r, g, b: f32 };
    ISize3  { x, y, z: isize } <-> Color64 { r, g, b: f64 };
    USize3  { x, y, z: usize } <-> Float3  { x, y, z: f32 };
    USize3  { x, y, z: usize } <-> Double3 { x, y, z: f64 };
    USize3  { x, y, z: usize } <-> Color   { r, g, b: f32 };
    USize3  { x, y, z: usize } <-> Color64 { r, g, b: f64 };
    Float3  { x, y, z: f32 }   <-> Double3 { x, y, z: f64 };
    Float3  { x, y, z: f32 }   <-> Color   { r, g, b: f32 };
    Float3  { x, y, z: f32 }   <-> Color64 { r, g, b: f64 };
    Double3 { x, y, z: f64 }   <-> Color   { r, g, b: f32 };
    Double3 { x, y, z: f64 }   <-> Color64 { r, g, b: f64 };
    Color   { r, g, b: f32 }   <-> Color64 { r, g, b: f64 };
}

impl Float2 {
    /// Gives vector rotated clockwise.
    pub fn rotate_clockwise(self) -> Self {
        Self::new(self.y, -self.x)
    }

    /// Gives vector rotated clockwise.
    pub fn rotate_counter_clockwise(self) -> Self {
        -self.rotate_clockwise()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let i = Float3::new(1.0, 0.0, 0.0);
        let j = Float3::new(0.0, 1.0, 0.0);
        let k = Float3::new(0.0, 0.0, 1.0);
        assert_eq!(i.cross(j), k);
    }

    #[test]
    fn test2() {
        let v1 = Int2::new(2, 0);
        let v2 = USize2::new(2, 3);
        let area = 2 * 3;
        assert_eq!(v1.cross(v2.into()),  area);
        assert_eq!(Int2::from(v2).cross(v1), -area);
    }

    #[test]
    fn swizzle_test1() {
        let v1 = Int3::new(1, 2, 3);
        let v2 = v1.zxx();
        assert_eq!(v2, Int3::new(3, 1, 1));
    }
}

#[cfg(test)]
mod impl_generator {
    #[derive(Clone, Copy, Debug)]
    struct VecType3D(&'static str, &'static str, &'static str, &'static str, &'static str);

    #[derive(Clone, Copy, Debug)]
    struct VecType2D(&'static str, &'static str, &'static str, &'static str);
    
    const TYPES_3D: [VecType3D; 16] = [
        VecType3D("Byte3",   "x", "y", "z", "i8"),
        VecType3D("UByte3",  "x", "y", "z", "u8"),
        VecType3D("Short3",  "x", "y", "z", "i16"),
        VecType3D("UShort3", "x", "y", "z", "u16"),
        VecType3D("Int3",    "x", "y", "z", "i32"),
        VecType3D("UInt3",   "x", "y", "z", "u32"),
        VecType3D("Long3",   "x", "y", "z", "i64"),
        VecType3D("ULong3",  "x", "y", "z", "u64"),
        VecType3D("Large3",  "x", "y", "z", "i128"),
        VecType3D("ULarge3", "x", "y", "z", "u128"),
        VecType3D("ISize3",  "x", "y", "z", "isize"),
        VecType3D("USize3",  "x", "y", "z", "usize"),
        VecType3D("Float3",  "x", "y", "z", "f32"),
        VecType3D("Double3", "x", "y", "z", "f64"),
        VecType3D("Color",   "r", "g", "b", "f32"),
        VecType3D("Color64", "r", "g", "b", "f64"),
    ];
    
    const TYPES_2D: [VecType2D; 14] = [
        VecType2D("Byte2",   "x", "y", "i8"),
        VecType2D("UByte2",  "x", "y", "u8"),
        VecType2D("Short2",  "x", "y", "i16"),
        VecType2D("UShort2", "x", "y", "u16"),
        VecType2D("Int2",    "x", "y", "i32"),
        VecType2D("UInt2",   "x", "y", "u32"),
        VecType2D("Long2",   "x", "y", "i64"),
        VecType2D("ULong2",  "x", "y", "u64"),
        VecType2D("Large2",  "x", "y", "i128"),
        VecType2D("ULarge2", "x", "y", "u128"),
        VecType2D("ISize2",  "x", "y", "isize"),
        VecType2D("USize2",  "x", "y", "usize"),
        VecType2D("Float2",  "x", "y", "f32"),
        VecType2D("Double2", "x", "y", "f64"),
    ];

    fn make_3_component_impl_string() -> String {
        TYPES_3D.iter()
            .map(|VecType3D(name, x, y, z, ty)| format!("pub {name} = ({x}, {y}, {z}): {ty};"))
            .fold(Default::default(), |lhs, rhs| lhs + "\n" + &rhs)
    }

    fn make_2_component_impl_string() -> String {
        TYPES_2D.iter()
            .map(|VecType2D(name, x, y, ty)| format!("pub {name} = ({x}, {y}): {ty};"))
            .fold(Default::default(), |lhs, rhs| lhs + "\n" + &rhs)
    }

    fn make_3_component_froms() -> String {
        const N_FROMS: usize = TYPES_3D.len() * (TYPES_3D.len() - 1) / 2;
        let mut froms = Vec::with_capacity(N_FROMS);

        for i in 0..TYPES_3D.len() {
        for j in i + 1 .. TYPES_3D.len() {
            let (VecType3D(name1, x1, y1, z1, type1), VecType3D(name2, x2, y2, z2, type2)) = (TYPES_3D[i], TYPES_3D[j]);
            froms.push(format!(
                "{name1} {{ {x1}, {y1}, {z1}: {type1} }} <-> {name2} {{ {x2}, {y2}, {z2}: {type2} }};"
            ));
        }}

        froms.into_iter()
            .fold(Default::default(), |lhs, rhs| lhs + "\n" + &rhs)
    }

    fn make_2_component_froms() -> String {
        const N_FROMS: usize = TYPES_2D.len() * (TYPES_2D.len() - 1) / 2;
        let mut froms = Vec::with_capacity(N_FROMS);

        for i in 0..TYPES_2D.len() {
        for j in i + 1 .. TYPES_2D.len() {
            let (VecType2D(name1, x1, y1, type1), VecType2D(name2, x2, y2, type2)) = (TYPES_2D[i], TYPES_2D[j]);
            froms.push(format!(
                "{name1} {{ {x1}, {y1}: {type1} }} <-> {name2} {{ {x2}, {y2}: {type2} }};"
            ));
        }}

        froms.into_iter()
            .fold(Default::default(), |lhs, rhs| lhs + "\n" + &rhs)
    }

    fn make_xyz_chooses_string() -> Vec<[char; 3]> {
        const NAMES: [char; 3] = ['x', 'y', 'z'];
        const N_CHOOSES: usize = NAMES.len().pow(NAMES.len() as u32);

        let mut result = Vec::with_capacity(N_CHOOSES);

        for &c1 in NAMES.iter() {
        for &c2 in NAMES.iter() {
        for &c3 in NAMES.iter() {
            result.push([c1, c2, c3]);
        }}}

        return result
    }

    fn make_xyz_2d_chooses_string() -> Vec<[char; 2]> {
        const NAMES: [char; 3] = ['x', 'y', 'z'];
        const N_CHOOSES: usize = NAMES.len().pow(2);

        let mut result = Vec::with_capacity(N_CHOOSES);

        for &c1 in NAMES.iter() {
        for &c2 in NAMES.iter() {
            result.push([c1, c2]);
        }}

        return result
    }

    fn make_3d_swizzles_string() -> String {
        let iter1 = make_xyz_chooses_string()
            .into_iter()
            .map(|[x, y, z]|
                format!(
                    "$vis fn {name}(self) -> Self {{ Self::new(self.${x}, self.${y}, self.${z}) }}",
                    name = String::from_iter([x, y, z]),
                )
            );

        let iter2 = make_xyz_2d_chooses_string()
            .into_iter()
            .map(|[x, y]|
                format!(
                    "$vis fn {name}(self) -> $Lower {{ $Lower::new(self.${x}, self.${y}) }}",
                    name = String::from_iter([x, y]),
                )
            );

        iter1.chain(iter2)
            .fold(String::default(), |lhs, rhs| lhs + "\n" + &rhs)
    }

    #[test]
    fn print_impl_3d() {
        println!("{}", make_3_component_impl_string());
    }

    #[test]
    fn print_froms_3d() {
        println!("{}",  make_3_component_froms());
    }

    #[test]
    fn print_impl_2d() {
        println!("{}", make_2_component_impl_string());
    }

    #[test]
    fn print_froms_2d() {
        println!("{}",  make_2_component_froms());
    }

    #[test]
    fn print_3d_swizzles() {
        println!("{}", make_3d_swizzles_string());
    }
}

impl Into<FXMVECTOR> for Float3 {
    fn into(self) -> FXMVECTOR {
        directx_math::XMVectorSet(self.x, self.y, self.z, 1.0)
    }
}

/// Represents 4D 32-bit float vector.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "byte_muck", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct Float4 {
    pub i_vec: XMVECTOR
}

#[allow(dead_code)]
impl Float4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { i_vec: XMVectorSet(x, y, z, w) }
    }

    pub fn x(&self) -> f32 { XMVectorGetX(self.i_vec) }
    pub fn y(&self) -> f32 { XMVectorGetY(self.i_vec) }
    pub fn z(&self) -> f32 { XMVectorGetZ(self.i_vec) }
    pub fn w(&self) -> f32 { XMVectorGetW(self.i_vec) }

    pub fn set_x(&mut self, x: f32) { self.i_vec = XMVectorSetX(self.i_vec, x) }
    pub fn set_y(&mut self, y: f32) { self.i_vec = XMVectorSetY(self.i_vec, y) }
    pub fn set_z(&mut self, z: f32) { self.i_vec = XMVectorSetZ(self.i_vec, z) }
    pub fn set_w(&mut self, w: f32) { self.i_vec = XMVectorSetW(self.i_vec, w) }

    /// Constructs vector from one number.
    pub fn all(xyzw: f32) -> Self {
        Self::new(xyzw, xyzw, xyzw, xyzw)
    }

    /// Constructs vector of ones.
    pub fn ones() -> Self {
        Self::all(1.0)
    }

    /// Constructs vector of zeros.
    pub fn zero() -> Self {
        Self::all(0.0)
    }

    /// Constructs vector from 3 floats and make W to be 1.0
    pub fn xyz1(x: f32, y: f32, z: f32) -> Self {
        Self::new(x, y, z, 1.0)
    }

    /// Constructs vector from 3 floats and make W to be 0.0
    pub fn xyz0(x: f32, y: f32, z: f32) -> Self {
        Self::new(x, y, z, 0.0)
    }
    
    /// Normalyzes the vector.
    pub fn normalyzed(self) -> Self {
        Float4 {
            i_vec: XMVector3Normalize(self.i_vec)
        }
    }

    /// Gives dot product of two vectors
    pub fn dot(self, other: Float4) -> f32 {
        XMVectorGetX(XMVector3Dot(self.i_vec, other.i_vec))
    }

    /// Gives cross product of two vectors
    pub fn cross(self, other: Float4) -> Self {
        Float4 {
            i_vec: XMVector3Cross(self.i_vec, other.i_vec)
        }
    }

    /// Gives component-vise absolute value vector.
    pub fn abs(self) -> Self {
        Self::new(self.x().abs(), self.y().abs(), self.z().abs(), self.w().abs())
    }

    /// Gives length value of vector
    pub fn len(self) -> f32 {
        XMVectorGetX(XMVector3Length(self.i_vec))
    }

    /// Represents [`Float4`] as an array.
    pub fn as_array(self) -> [f32; 4] {
        [self.x(), self.y(), self.z(), self.w()]
    }

    /// Represents [`Float4`] as a tuple.
    pub fn as_tuple(self) -> (f32, f32, f32, f32) {
        (self.x(), self.y(), self.z(), self.w())
    }
}

impl Default for Float4 {
    fn default() -> Self {
        Self::all(0.0)
    }
}

impl From<Int3> for Float4 {
    fn from(p: Int3) -> Self {
        Self::xyz0(p.x() as f32, p.y() as f32, p.z() as f32)
    }
}

impl From<Float4> for Int3 {
    fn from(p: Float4) -> Self {
        Self::new(p.x() as i32, p.y() as i32, p.z() as i32)
    }
}

impl From<Float3> for Float4 {
    fn from(p: Float3) -> Self {
        Self::new(p.x, p.y, p.z, 1.0)
    }
}

impl PartialEq for Float4 {
    fn eq(&self, other: &Self) -> bool {
        self.x() == other.x() &&
        self.y() == other.y() &&
        self.z() == other.z() &&
        self.w() == other.w()
    }
    fn ne(&self, other: &Self) -> bool {
        self.x() != other.x() ||
        self.y() != other.y() ||
        self.z() != other.z() ||
        self.w() != other.w()
    }
}

impl std::ops::Neg for Float4 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x(), -self.y(), -self.z(), -self.w())
    }
}

impl std::ops::Sub for Float4 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(
            self.x() - other.x(),
            self.y() - other.y(),
            self.z() - other.z(),
            self.w() - other.w()
        )
    }
}

impl std::ops::SubAssign for Float4 {
    fn sub_assign(&mut self, other: Self) {
        self.set_x(self.x() - other.x());
        self.set_y(self.y() - other.y());
        self.set_z(self.z() - other.z());
        self.set_w(self.w() - other.w());
    }
}

impl std::ops::Add for Float4 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(
            self.x() + other.x(),
            self.y() + other.y(),
            self.z() + other.z(),
            self.w() + other.w()
        )
    }
}

impl std::ops::AddAssign for Float4 {
    fn add_assign(&mut self, other: Self) {
        self.set_x(self.x() + other.x());
        self.set_y(self.y() + other.y());
        self.set_z(self.z() + other.z());
        self.set_w(self.w() + other.w());
    }
}

impl std::ops::Mul<f32> for Float4 {
    type Output = Self;
    fn mul(self, k: f32) -> Self {
        Self::new(self.x() * k, self.y() * k, self.z() * k, self.w() * k)
    }
}

impl std::ops::Mul for Float4 {
    type Output = Self;
    fn mul(self, p: Self) -> Self {
        Self::new(self.x() * p.x(), self.y() * p.y(), self.z() * p.z(), self.w() * p.w())
    }
}

impl std::ops::MulAssign<f32> for Float4 {
    fn mul_assign(&mut self, k: f32) {
        self.set_x(self.x() * k);
        self.set_y(self.y() * k);
        self.set_z(self.z() * k);
        self.set_w(self.w() * k);
    }
}

impl std::ops::MulAssign for Float4 {
    fn mul_assign(&mut self, p: Self) {
        self.set_x(self.x() * p.x());
        self.set_y(self.y() * p.y());
        self.set_z(self.z() * p.z());
        self.set_w(self.w() * p.w());
    }
}

impl std::ops::Div<f32> for Float4 {
    type Output = Self;
    fn div(self, k: f32) -> Self {
        Self::new(self.x() / k, self.y() / k, self.z() / k, self.w() / k)
    }
}

impl std::ops::Div for Float4 {
    type Output = Self;
    fn div(self, k: Self) -> Self {
        Self::new(self.x() / k.x(), self.y() / k.y(), self.z() / k.z(), self.w() / k.w())
    }
}

impl std::ops::DivAssign<f32> for Float4 {
    fn div_assign(&mut self, k: f32) {
        assert_ne!(k, 0.0, "Cannot divide by 0!");
        self.set_x(self.x() / k);
        self.set_y(self.y() / k);
        self.set_z(self.z() / k);
        self.set_w(self.w() / k);
    }
}

impl std::ops::DivAssign for Float4 {
    fn div_assign(&mut self, k: Self) {
        self.set_x(self.x() / k.x());
        self.set_y(self.y() / k.y());
        self.set_z(self.z() / k.z());
        self.set_w(self.w() / k.w());
    }
}
