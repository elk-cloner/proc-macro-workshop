use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;

use syn::{parse_macro_input, DeriveInput};

fn get_debug_attribute(field: &syn::Field) -> syn::Result<Option<proc_macro2::Literal>> {
    let mut debug_attr = None;

    for attr in &field.attrs {
        let meta = match &attr.meta {
            syn::Meta::NameValue(meta) if meta.path.is_ident("debug") => meta,
            _ => continue,
        };

        if debug_attr.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate #[debug] attribute",
            ));
        }

        let string_literal = match &meta.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => s,
            _ => {
                return Err(syn::Error::new_spanned(
                    &meta.value,
                    "debug attribute value must be a string literal",
                ))
            }
        };
        debug_attr = Some(string_literal.token());
    }

    Ok(debug_attr)
}

fn generate_debug_bounds(
    param: &syn::GenericParam,
    phantom_only_types: &HashSet<&syn::Ident>,
) -> Option<proc_macro2::TokenStream> {
    let type_param = match param {
        syn::GenericParam::Type(tp) => tp,
        _ => return None,
    };

    if phantom_only_types.contains(&type_param.ident) {
        return None;
    }

    let ident = &type_param.ident;
    Some(quote! { #ident: ::std::fmt::Debug })
}

fn extract_phantom_type_params(field: &syn::Field) -> Vec<&syn::Ident> {
    let syn::Type::Path(type_path) = &field.ty else {
        return Vec::new();
    };

    let Some(last_segment) = type_path.path.segments.last() else {
        return Vec::new();
    };

    if last_segment.ident != "PhantomData" {
        return Vec::new();
    }

    let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments else {
        return Vec::new();
    };

    args.args
        .iter()
        .filter_map(|arg| match arg {
            syn::GenericArgument::Type(syn::Type::Path(tp)) => tp.path.get_ident(),
            _ => None,
        })
        .collect()
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

    let phantom_only_types: HashSet<&syn::Ident> = fields
        .iter()
        .flat_map(extract_phantom_type_params)
        .collect();

    let debug_bounds: Vec<_> = ast
        .generics
        .params
        .iter()
        .filter_map(|param| generate_debug_bounds(param, &phantom_only_types))
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
