use proc_macro::TokenStream;
use quote::quote;

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

fn generate_debug_bounds(param: &syn::GenericParam) -> Option<proc_macro2::TokenStream> {
    match param {
        syn::GenericParam::Type(type_param) => {
            let ident = &type_param.ident;
            return Some(quote! {#ident: ::std::fmt::Debug});
        }
        _ => None,
    }
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

    let debug_bounds: Vec<_> = ast
        .generics
        .params
        .iter()
        .filter_map(generate_debug_bounds)
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
