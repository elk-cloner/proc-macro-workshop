use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Field, Type};

fn extract_option_inner_type(field: &Field) -> Option<&Type> {
    let path = match &field.ty {
        Type::Path(type_path) => &type_path.path,
        _ => return None,
    };

    let segment = path.segments.first()?;
    if segment.ident != "Option" || path.segments.len() != 1 {
        return None;
    }

    let args = match &segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args,
        _ => return None,
    };

    match args.args.first()? {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

fn is_option_type(field: &Field) -> bool {
    extract_option_inner_type(field).is_some()
}

fn generate_builder_field(field: &Field) -> proc_macro2::TokenStream {
    let name = &field.ident;
    let ty = &field.ty;

    if is_option_type(field) {
        quote! { #name: #ty }
    } else {
        quote! { #name: ::std::option::Option<#ty> }
    }
}

fn generate_setter_method(field: &Field) -> proc_macro2::TokenStream {
    let name = &field.ident;

    if let Some(inner_ty) = extract_option_inner_type(field) {
        // For Option<T> fields, setter takes T
        quote! {
            pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                self.#name = ::std::option::Option::Some(#name);
                self
            }
        }
    } else {
        // For T fields, setter takes T
        let ty = &field.ty;
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = ::std::option::Option::Some(#name);
                self
            }
        }
    }
}

fn generate_build_field(field: &Field) -> proc_macro2::TokenStream {
    let name = &field.ident;

    if is_option_type(field) {
        // Optional fields: use the Option value directly
        quote! {
            #name: self.#name.clone()
        }
    } else {
        // Required fields: unwrap with error message
        quote! {
            #name: self.#name.clone()
                .ok_or_else(|| ::std::boxed::Box::<dyn ::std::error::Error>::from(
                    ::std::format!("field `{}` is not set", ::std::stringify!(#name))
                ))?
        }
    }
}

fn generate_empty_field(field: &Field) -> proc_macro2::TokenStream {
    let name = &field.ident;
    quote! { #name: ::std::option::Option::None }
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", struct_name), struct_name.span());

    // Extract named fields from struct
    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => {
            return syn::Error::new_spanned(
                &ast,
                "Builder can only be derived for structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate code sections
    let builder_fields = fields.iter().map(generate_builder_field);
    let setter_methods = fields.iter().map(generate_setter_method);
    let build_fields = fields.iter().map(generate_build_field);
    let empty_fields = fields.iter().map(generate_empty_field);

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl #builder_name {
            #(#setter_methods)*

            pub fn build(&mut self) -> ::std::result::Result<#struct_name, ::std::boxed::Box<dyn ::std::error::Error>> {
                ::std::result::Result::Ok(#struct_name {
                    #(#build_fields,)*
                })
            }
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#empty_fields,)*
                }
            }
        }
    };

    expanded.into()
}
