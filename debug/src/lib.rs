use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;

use syn::{parse_macro_input, DeriveInput};

fn get_debug_attribute(field: &syn::Field) -> syn::Result<Option<proc_macro2::Literal>> {
    for attr in &field.attrs {
        if let syn::Meta::NameValue(meta) = &attr.meta {
            if meta.path.is_ident("debug") {
                match &meta.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) => {
                        return Ok(Some(s.token()));
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &meta.value,
                            "debug attribute must be a string literal",
                        ))
                    }
                }
            }
        }
    }
    Ok(None)
}

fn generate_debug_bounds(
    param: &syn::GenericParam,
    phantomdata_generics: &HashSet<String>,
) -> Option<proc_macro2::TokenStream> {
    match param {
        syn::GenericParam::Type(type_param) => {
            let ident = &type_param.ident;
            if phantomdata_generics.contains(&ident.to_string()) {
                return None;
            } else {
                return Some(quote! {#ident: ::std::fmt::Debug});
            }
        }
        _ => None,
    }
}

fn extract_phantom_generics(field: &syn::Field) -> Option<HashSet<String>> {
    let path = match &field.ty {
        syn::Type::Path(type_path) => &type_path.path,
        _ => return None,
    };

    let segment = path.segments.first()?;
    if segment.ident != "PhantomData" {
        return None;
    }

    let args = match &segment.arguments {
        syn::PathArguments::AngleBracketed(arguments) => &arguments.args,
        _ => {
            return None;
        }
    };

    let mut phantom_types = HashSet::new();
    for arg in args.iter() {
        match arg {
            syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path, .. })) => {
                if let Some(ident) = path.get_ident() {
                    phantom_types.insert(ident.to_string());
                }
            }
            _ => {}
        }
    }
    return Some(phantom_types);
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", ast);

    let struct_name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => {
            return syn::Error::new_spanned(
                &ast,
                "CustomDebug only supports structs with named fields",
            )
            .into_compile_error()
            .into()
        }
    };

    let fmt_field_calls = fields.iter().map(|field| {
        let f_name = &field.ident;

        match get_debug_attribute(field) {
            Ok(Some(format_string)) => {
                quote! { .field(stringify!(#f_name), &format_args!(#format_string, &self.#f_name))}
            }
            Ok(None) => {
                quote! { .field(stringify!(#f_name), &self.#f_name)}
            }
            Err(err) => {
                // Return error as compile_error! token
                err.to_compile_error()
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let phantomdata_generics: HashSet<String> = fields
        .iter()
        .filter_map(extract_phantom_generics)
        .flatten()
        .collect();

    let debug_bounds: Vec<_> = ast
        .generics
        .params
        .iter()
        .filter_map(|param| generate_debug_bounds(param, &phantomdata_generics))
        .collect();

    let expanded = quote! {
        impl #impl_generics ::std::fmt::Debug for #struct_name #ty_generics
        where
            #(#debug_bounds,)*
            #where_clause
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(stringify!(#struct_name))
                    #(#fmt_field_calls)*
                    .finish()
            }
        }
    };

    expanded.into()
}
