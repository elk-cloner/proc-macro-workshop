use proc_macro::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::parse::Parse;
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
    associated_types: &HashMap<&syn::Ident, &syn::Ident>,
) -> Option<proc_macro2::TokenStream> {
    let type_param = match param {
        syn::GenericParam::Type(tp) => tp,
        _ => return None,
    };

    if phantom_only_types.contains(&type_param.ident) {
        return None;
    }

    let ident = &type_param.ident;
    match associated_types.get(ident) {
        Some(value) => Some(quote! { #ident::#value: ::std::fmt::Debug }),
        _ => Some(quote! { #ident: ::std::fmt::Debug }),
    }
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

fn get_associated_types(field: &syn::Field) -> HashMap<&syn::Ident, &syn::Ident> {
    let syn::Type::Path(type_path) = &field.ty else {
        return HashMap::new();
    };

    let Some(segment) = type_path.path.segments.last() else {
        return HashMap::new();
    };

    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return HashMap::new();
    };

    let Some(arg) = arguments.args.last() else {
        return HashMap::new();
    };

    let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path, .. })) = arg else {
        return HashMap::new();
    };

    let Some(key) = path.segments.first() else {
        return HashMap::new();
    };
    let Some(value) = path.segments.last() else {
        return HashMap::new();
    };

    HashMap::from([(&key.ident, &value.ident)])
}

struct BoundAttr {
    value: String,
}

impl Parse for BoundAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _bound: syn::Ident = input.parse()?;
        let _eq: syn::Token![=] = input.parse()?;
        let value: syn::LitStr = input.parse()?;

        Ok(BoundAttr {
            value: value.value(),
        })
    }
}

fn get_manual_bounds(ast: &DeriveInput) -> syn::Result<Option<syn::WhereClause>> {
    for attr in &ast.attrs {
        if attr.path().is_ident("debug") {
            let bound_attr: BoundAttr = attr.parse_args()?;
            let where_clause: syn::WhereClause =
                syn::parse_str(&format!("where {}", bound_attr.value))?;
            return Ok(Some(where_clause));
        }
    }
    Ok(None)
}

fn extract_type_params_from_manual_bounds(
    manual_bounds: &syn::WhereClause,
) -> HashSet<&syn::Ident> {
    let mut covered_types = HashSet::new();

    for predicate in &manual_bounds.predicates {
        if let syn::WherePredicate::Type(type_predicate) = predicate {
            if let syn::Type::Path(type_path) = &type_predicate.bounded_ty {
                if let Some(first_segment) = type_path.path.segments.first() {
                    covered_types.insert(&first_segment.ident);
                }
            }
        }
    }

    covered_types
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let manual_bounds = match get_manual_bounds(&ast) {
        Ok(bounds) => bounds,
        Err(err) => return err.to_compile_error().into(),
    };

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
            Err(err) => err.to_compile_error(),
        }
    });

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let manually_bounded_types = manual_bounds
        .as_ref()
        .map(extract_type_params_from_manual_bounds)
        .unwrap_or_default();

    let associated_types = fields.iter().flat_map(get_associated_types).collect();
    let phantom_only_types: HashSet<&syn::Ident> = fields
        .iter()
        .flat_map(extract_phantom_type_params)
        .collect();

    let auto_bounds: Vec<_> = ast
        .generics
        .params
        .iter()
        .filter_map(|param| {
            if let syn::GenericParam::Type(type_param) = param {
                if manually_bounded_types.contains(&type_param.ident) {
                    return None;
                }
            }
            generate_debug_bounds(param, &phantom_only_types, &associated_types)
        })
        .collect();

    let manual_predicates = manual_bounds
        .as_ref()
        .map(|mb| &mb.predicates)
        .into_iter()
        .flatten();

    let expanded = quote! {
        impl #impl_generics ::std::fmt::Debug for #struct_name #ty_generics
        where
            #(#auto_bounds,)*
            #(#manual_predicates,)*
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
