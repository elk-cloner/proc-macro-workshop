use proc_macro::TokenStream;
use quote::quote;
use syn::{parse2, parse_macro_input, spanned::Spanned, DeriveInput, Field, Type};

fn extract_inner_type<'a>(field: &'a Field, ident_type: &str) -> Option<&'a Type> {
    let path = match &field.ty {
        Type::Path(type_path) => &type_path.path,
        _ => return None,
    };

    let segment = path.segments.first()?;
    if segment.ident != ident_type || path.segments.len() != 1 {
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
    extract_inner_type(field, "Option").is_some()
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

fn get_each_attribute_clean(field: &Field) -> Option<String> {
    for attr in &field.attrs {
        let list = match &attr.meta {
            syn::Meta::List(list) if list.path.is_ident("builder") => list,
            _ => continue,
        };
        let parsed: syn::Result<EachAttr> = parse2(list.tokens.clone());
        if let Ok(each_attr) = parsed {
            return Some(each_attr.value);
        }
    }
    None
}

struct EachAttr {
    value: String,
}
impl syn::parse::Parse for EachAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "each" {
            return Err(syn::Error::new_spanned(ident, "expected `each`"));
        }

        let _eq: syn::Token![=] = input.parse()?;
        let value: syn::LitStr = input.parse()?;

        Ok(EachAttr {
            value: value.value(),
        })
    }
}

fn generate_setter_method(field: &Field) -> proc_macro2::TokenStream {
    let name = &field.ident;

    if let Some(inner_ty) = extract_inner_type(&field, "Option") {
        quote! {
            pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    } else if let Some(each_attr) = get_each_attribute_clean(&field) {
        let each_method = syn::Ident::new(&each_attr, field.span());
        let inner_type =
            extract_inner_type(&field, "Vec").expect("fields with 'each' attribute must be Vec<T>");

        quote! {
            pub fn #each_method(&mut self, #each_method: #inner_type) -> &mut Self {
                self.#name.get_or_insert_with(Vec::new).push(#each_method);
                self
            }
        }
    } else {
        let ty = &field.ty;
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
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
        match get_each_attribute_clean(&field) {
            Some(_) => {
                quote! {
                    #name: self.#name.clone().unwrap_or_default()
                }
            }
            _ => {
                quote! {
                    #name: self.#name.clone()
                        .ok_or_else(|| ::std::boxed::Box::<dyn ::std::error::Error>::from(
                            ::std::format!("field `{}` is not set", ::std::stringify!(#name))
                        ))?
                }
            }
        }
    }
}

fn generate_empty_field(field: &Field) -> proc_macro2::TokenStream {
    let name = &field.ident;
    quote! { #name: ::std::option::Option::None }
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", struct_name), struct_name.span());
    // println!("{:#?}", ast);

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
