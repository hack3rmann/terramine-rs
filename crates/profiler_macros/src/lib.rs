use proc_macro::*;
use syn::*;
use quote::quote;

#[proc_macro_attribute]
pub fn profiler_target(_args: TokenStream, item: TokenStream) -> TokenStream {
    let id: u64 = rand::random();

    let input_fn = parse_macro_input!(item as ItemFn);
    let block = input_fn.block;
    let sig = input_fn.sig;
    let vis = input_fn.vis;

    quote! {
        #vis #sig {
            static __id: MeasureId = #id;
            let __measure = profiler::start_capture(stringify!(#sig), __id);
            #block
        }
    }.into()
}