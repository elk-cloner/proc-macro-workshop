use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn extract_option_type(f: &syn::Field) -> Option<&syn::Type> {
    let path = match &f.ty {
        syn::Type::Path(type_path) => &type_path.path,
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

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let bname = format!("{}Builder", name);
    let bident = syn::Ident::new(&bname, name.span());
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };

    let ty_is_option = |f: &syn::Field| {
        if let syn::Type::Path(ref p) = f.ty {
            return p.path.segments.len() == 1 && p.path.segments[0].ident == "Option";
        }
        false
    };

    let optionized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_is_option(&f) {
            quote! { #name: #ty }
        } else {
            quote! { #name: std::option::Option<#ty> }
        }
    });

    let methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_is_option(&f) {
            let inner_ty = extract_option_type(&f);
            quote! {
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self{
                    self.#name = Some(#name);
                    self
                }
            }
        } else {
            quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self{
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if ty_is_option(&f) {
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), "is not set"))?
            }
        }
    });

    let build_empty = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: None }
    });

    let expanded = quote! {
        struct #bident {
            #(#optionized,)*
        }
        impl #bident {
            #(#methods)*
            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
        impl #name {
            fn builder() -> #bident {
                #bident{
                    #(#build_empty,)*
                }
            }
        }
    };
    expanded.into()
}
