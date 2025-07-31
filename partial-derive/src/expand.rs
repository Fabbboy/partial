use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DataStruct, DeriveInput, Error, Fields};

const PARTIAL_ATTRIBUTE: &str = "partial";

fn collect_fields(data: &DataStruct, only_marked: bool) -> Vec<syn::Field> {
    let has_marker = |field: &syn::Field| {
        field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident(PARTIAL_ATTRIBUTE))
    };

    match &data.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .filter(|f| has_marker(f) == only_marked)
            .cloned()
            .collect(),

        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .filter(|f| has_marker(f) == only_marked)
            .cloned()
            .collect(),

        syn::Fields::Unit => vec![],
    }
}

fn remove_marker_attributes(fields: &mut Vec<syn::Field>) {
    for field in fields.iter_mut() {
        field
            .attrs
            .retain(|attr| !attr.path().is_ident(PARTIAL_ATTRIBUTE));
    }
}

fn name_unpatched_struct(ident: &Ident) -> Ident {
    let new_name = format!("{}Unpatched", ident);
    Ident::new(&new_name, ident.span())
}

fn extract_field_types(fields: &[syn::Field]) -> Vec<syn::Type> {
    fields.iter().map(|f| f.ty.clone()).collect()
}

fn expand_patched_impl(ast: &DeriveInput) -> TokenStream {
    let generics = &ast.generics;
    let struct_name = &ast.ident;

    quote! {
        unsafe impl #generics ::partial::marker::Patched for #struct_name #generics  {}
    }
}

fn expand_patchable_impl(ast: &DeriveInput, st: &DataStruct) -> TokenStream {
    let struct_name = &ast.ident;
    let unpatched_name = name_unpatched_struct(&ast.ident);
    let generics = &ast.generics;

    let all_fields: Vec<_> = match &st.fields {
        Fields::Named(fields) => fields.named.iter().collect(),
        Fields::Unnamed(fields) => fields.unnamed.iter().collect(),
        Fields::Unit => vec![],
    };

    let partial_fields: Vec<_> = all_fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident(PARTIAL_ATTRIBUTE)))
        .cloned()
        .collect();

    let arg_types: Vec<_> = partial_fields.iter().map(|f| f.ty.clone()).collect();
    let mut arg_indices = HashMap::new();
    for (i, f) in partial_fields.iter().enumerate() {
        if let Some(ident) = f.ident.clone() {
            arg_indices.insert(ident.to_string(), syn::Index::from(i));
        }
    }

    match &st.fields {
        Fields::Named(_) => {
            let assignments = all_fields.iter().map(|f| {
                let ident = f.ident.as_ref().unwrap();
                if f.attrs.iter().any(|a| a.path().is_ident(PARTIAL_ATTRIBUTE)) {
                    let idx = &arg_indices[&ident.to_string()];
                    quote! { #ident: args.#idx }
                } else {
                    quote! { #ident: self.#ident }
                }
            });

            quote! {
                unsafe impl #generics ::partial::patch::Patchable for #unpatched_name #generics {
                    type Args = ((#(#arg_types,)*));
                    type Patched = #struct_name #generics;

                    fn patch(&self, args: Self::Args) -> Self::Patched {
                        #struct_name {
                            #(#assignments),*
                        }
                    }
                }
            }
        }
        Fields::Unnamed(_) => {
            let mut arg_index = 0_usize;
            let mut self_index = 0_usize;

            let assignments = all_fields.iter().map(|f| {
                if f.attrs.iter().any(|a| a.path().is_ident(PARTIAL_ATTRIBUTE)) {
                    let idx = syn::Index::from(arg_index);
                    arg_index += 1;
                    quote! { args.#idx }
                } else {
                    let idx = syn::Index::from(self_index);
                    self_index += 1;
                    quote! { self.#idx }
                }
            });

            quote! {
                unsafe impl #generics ::partial::patch::Patchable for #unpatched_name #generics {
                    type Args = (#(#arg_types,)*);
                    type Patched = #struct_name #generics;

                    fn patch(&self, args: Self::Args) -> Self::Patched {
                        #struct_name(#(#assignments),*)
                    }
                }
            }
        }

        Fields::Unit => {
            quote! {
                unsafe impl #generics ::partial::patch::Patchable for #unpatched_name #generics {
                    type Args = ();
                    type Patched = #struct_name #generics;

                    fn patch(&self, _args: Self::Args) -> Self::Patched {
                        #struct_name
                    }
                }
            }
        }
    }
}

fn expand_unpatched_struct(ast: &DeriveInput, st: &DataStruct) -> TokenStream {
    let struct_name = &ast.ident;
    let generics = &ast.generics;
    let mut fields = collect_fields(st, false);
    remove_marker_attributes(&mut fields);
    let vis = &ast.vis;
    let unpatched_name = name_unpatched_struct(struct_name);

    match &st.fields {
        Fields::Named(_) => {
            let field_idents: Vec<_> = fields
                .iter()
                .map(|f| f.ident.as_ref().expect("Expected named field"))
                .collect();
            let field_types: Vec<_> = extract_field_types(&fields);

            quote! {
                #vis struct #unpatched_name #generics {
                   #(#field_idents: #field_types),*
                }

                unsafe impl #generics ::partial::marker::Unpatched for #unpatched_name #generics {}
            }
        }

        Fields::Unnamed(_) => {
            let field_types: Vec<_> = extract_field_types(&fields);

            quote! {
                #vis struct #unpatched_name #generics (
                    #(#field_types),*
                );

                unsafe impl #generics ::partial::marker::Unpatched for #unpatched_name #generics {}
            }
        }

        Fields::Unit => {
            quote! {
                #vis struct #unpatched_name;

                unsafe impl #generics ::partial::marker::Unpatched for #unpatched_name #generics {}
            }
        }
    }
}

pub fn expand_partial(ast: DeriveInput) -> TokenStream {
    let struct_data = match &ast.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Error::new_spanned(ast, "Partial can only be derived for structs")
                .to_compile_error();
        }
    };

    let patched_impl = expand_patched_impl(&ast);
    let unpatched_struct = expand_unpatched_struct(&ast, struct_data);
    let patchable_impl = expand_patchable_impl(&ast, struct_data);

    quote! {
        #patched_impl
        #unpatched_struct
        #patchable_impl
    }
}
