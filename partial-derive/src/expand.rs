use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DataStruct, DeriveInput, Error};

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
    let fields = collect_fields(st, true);
    let field_types = extract_field_types(&fields);

    let indices = (0..field_types.len()).map(syn::Index::from).collect::<Vec<_>>();


    match st.fields {
        syn::Fields::Named(_) => {
            let field_idents: Vec<_> = fields
                .iter()
                .map(|f| f.ident.as_ref().expect("Expected named field"))
                .collect();

            quote! {
                unsafe impl #generics ::partial::patch::Patchable for #unpatched_name #generics {
                    type Args = (#(#field_types),*);
                    type Patched = #struct_name #generics;

                    fn patch(&self, args: Self::Args) -> Self::Patched {
                        #struct_name {
                            #(#field_idents: args.#indices),*
                        }
                    }
                }
            }
        }
        _ => Error::new_spanned(
            ast,
            "Partial can only be derived for structs with named fields",
        )
        .to_compile_error(),
    }
}

fn expand_unpatched_struct(ast: &DeriveInput, st: &DataStruct) -> TokenStream {
    let struct_name = &ast.ident;
    let generics = &ast.generics;
    let mut fields = collect_fields(st, false);
    remove_marker_attributes(&mut fields);

    let field_idents: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("Expected named field"))
        .collect();
    let field_types = extract_field_types(&fields);

    let vis = &ast.vis;
    let unpatched_name = name_unpatched_struct(struct_name);

    quote! {
        #vis struct #unpatched_name #generics {
           #(#field_idents: #field_types),*
        }

        unsafe impl #generics ::partial::marker::Unpatched for #unpatched_name #generics {}
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
