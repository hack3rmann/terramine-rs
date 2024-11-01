use {
    proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2},
    proc_macro_crate::{crate_name, FoundCrate},
    quote::{quote, quote_spanned},
    syn::{spanned::Spanned, Error},
};



/// Implements the derive of `#[derive(ConstDefault)]` for struct types.
pub fn derive_default(input: TokenStream2) -> Result<TokenStream2, syn::Error> {
	let syn::DeriveInput {
        ident, mut generics, data, ..
    } = syn::parse2::<syn::DeriveInput>(input)?;

	let crate_ident = query_crate_ident()?;

	let syn::Data::Struct(data_struct) = data else {
        return Err(Error::new(
            Span::call_site(),
            "ConstDefault derive only works on struct types",
        ));
    };

	let default_struct = generate_struct(&crate_ident, &data_struct)?;

	generate_where_bounds(&crate_ident, &data_struct, &mut generics)?;

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	Ok(quote! {
		impl #impl_generics crate::app::utils::const_default::ConstDefault for #ident #ty_generics #where_clause {
			const DEFAULT: Self = #default_struct;
		}

        impl #impl_generics ::std::default::Default for #ident #ty_generics {
            fn default() -> Self { <Self as crate::app::utils::const_default::ConstDefault>::DEFAULT }
        }
	})
}

/// Queries the dependencies for the derive root crate name and returns the identifier.
///
/// # Note
///
/// This allows to use crate aliases in `Cargo.toml` files of dependencies.
fn query_crate_ident() -> Result<TokenStream2, syn::Error> {
	let query = crate_name("terramine").map_err(|err|
		Error::new(
			Span::call_site(),
			format!("could not find root crate for ConstDefault derive: {err}"),
		)
	)?;

	match query {
		FoundCrate::Itself => Ok(quote! { crate }),
		FoundCrate::Name(name) => {
			let ident = Ident::new(&name, Span::call_site());
			Ok(quote! { ::#ident })
		}
	}
}

/// Generates the `ConstDefault` implementation for `struct` input types.
///
/// # Note
///
/// The generated code abuses the fact that in Rust struct types can always
/// be represented with braces and either using identifiers for fields or
/// raw number literals in case of tuple-structs.
///
/// For example `struct Foo(u32)` can be represented as `Foo { 0: 42 }`.
fn generate_struct(
    _crate_ident: &TokenStream2, data_struct: &syn::DataStruct,
) -> Result<TokenStream2, syn::Error> {
	let fields_impl = data_struct.fields.iter()
        .enumerate()
        .map(|(n, field)| {
            let field_span = field.span();
            let field_type = &field.ty;
            let field_pos = Literal::usize_unsuffixed(n);
            let field_ident = field
                .ident
                .as_ref()
                .map(|ident| quote_spanned!(field_span=> #ident))
                .unwrap_or_else(|| quote_spanned!(field_span=> #field_pos));

            quote_spanned!(field_span=>
                #field_ident: <#field_type as crate::app::utils::const_default::ConstDefault>::DEFAULT
            )
        });

	Ok(quote! {
		Self { #(#fields_impl),* }
	})
}

/// Generates `ConstDefault` where bounds for all fields of the input.
fn generate_where_bounds(
	_crate_ident: &TokenStream2, data_struct: &syn::DataStruct, generics: &mut syn::Generics,
) -> Result<(), syn::Error> {
	let where_clause = generics.make_where_clause();

	for syn::Field { ref ty, .. } in &data_struct.fields {
		where_clause.predicates.push(syn::parse_quote!(
			#ty: crate::app::utils::const_default::ConstDefault
		))
	}

	Ok(())
}