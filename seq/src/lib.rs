use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Token};

#[derive(Debug)]
struct SeqMacroInput {
    var: syn::Ident,
    _in: syn::Token![in],
    range_start: syn::LitInt,
    range_end: syn::LitInt,
    is_inclusive: bool,
    body: TokenStream2,
}

impl Parse for SeqMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let var: syn::Ident = input.parse()?;
        let _in: syn::Token![in] = input.parse()?;
        let range_start: syn::LitInt = input.parse()?;
        let is_inclusive = if input.peek(syn::Token![..=]) {
            input.parse::<Token![..=]>()?;
            true
        } else {
            input.parse::<Token![..]>()?;
            false
        };
        let range_end: syn::LitInt = input.parse()?;
        // println!("{:?}", range_end);
        let content;
        syn::braced!(content in input );
        let body: TokenStream2 = content.parse()?;
        Ok(SeqMacroInput {
            var,
            _in,
            range_start,
            range_end,
            is_inclusive,
            body,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as SeqMacroInput);
    // println!("{:#?}", ast);
    quote! { /* .. */}.into()
}
