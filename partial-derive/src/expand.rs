use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand_partial(ast: DeriveInput) -> TokenStream {
    quote! {}.into()
}
