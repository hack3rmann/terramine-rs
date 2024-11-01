#![crate_type = "proc-macro"]

use proc_macro::TokenStream;



mod const_default;




/// Derives an implementation for the [`ConstDefault`] trait.
///
/// # Note
///
/// Currently only works with `struct` inputs.
///
/// # Example
///
/// ## Struct
///
/// ```
/// # use const_default::ConstDefault;
/// #[derive(ConstDefault)]
/// # #[derive(Debug, PartialEq)]
/// pub struct Color {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// assert_eq!(
///     <Color as ConstDefault>::DEFAULT,
///     Color { r: 0, g: 0, b: 0 },
/// )
/// ```
///
/// ## Tuple Struct
///
/// ```
/// # use const_default::ConstDefault;
/// #[derive(ConstDefault)]
/// # #[derive(Debug, PartialEq)]
/// pub struct Vec3(f32, f32, f32);
///
/// assert_eq!(
///     <Vec3 as ConstDefault>::DEFAULT,
///     Vec3(0.0, 0.0, 0.0),
/// )
/// ```
#[proc_macro_derive(ConstDefault, attributes(const_default))]
pub fn derive(input: TokenStream) -> TokenStream {
	match const_default::derive_default(input.into()) {
		Ok(output) => output.into(),
		Err(error) => error.to_compile_error().into(),
	}
}