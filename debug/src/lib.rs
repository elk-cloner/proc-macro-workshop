use proc_macro::TokenStream;
use quote::quote;

use syn::{parse_macro_input, DeriveInput};

fn get_debug_attribute(field: &syn::Field) -> Option<proc_macro2::Literal> {
    let attr = if field.attrs.len() != 0 {
        &field.attrs[0]
    } else {
        return None;
    };

    let meta = match &attr.meta {
        syn::Meta::NameValue(meta) => meta,
        _ => {
            return None;
        }
    };

    let meta_value = if meta.path.segments.len() > 0 && meta.path.segments[0].ident == "debug" {
        &meta.value
    } else {
        return None;
    };

    match &meta_value {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(str),
            ..
        }) => Some(str.token()),
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
            Some(format_string) => {
                quote! { .field(stringify!(#f_name), &format_args!(#format_string, &self.#f_name))}
            }
            _ => {
                quote! { .field(stringify!(#f_name), &self.#f_name)}
            }
        }
    });

    let expanded = quote! {
        impl ::std::fmt::Debug for #struct_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(stringify!(#struct_name))
                    #(#fmt_field_calls)*
                    .finish()
            }
        }
    };

    expanded.into()
}
