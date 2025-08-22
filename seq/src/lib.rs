use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::TokenStreamExt;
use syn::parse::{Parse, ParseStream};
use syn::{Token, parse_macro_input};

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

        let content;
        syn::braced!(content in input );
        let body: TokenStream2 = content.parse()?;
        // println!("{:?}", body);

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

fn find_and_replace_n(input: TokenStream2, n_value: u8) -> TokenStream2 {
    let input: Vec<proc_macro2::TokenTree> = input.into_iter().collect();
    let mut token_stream = TokenStream2::new();

    let mut i = 0;
    while i < input.len() {
        match &input[i] {
            proc_macro2::TokenTree::Group(group) => {
                let c = find_and_replace_n(group.stream(), n_value);
                let c =
                    proc_macro2::TokenTree::Group(proc_macro2::Group::new(group.delimiter(), c));
                token_stream.append(c);
                i += 1
            }
            proc_macro2::TokenTree::Ident(ident) => {
                // Look ahead for pattern: current_ident ~ N
                if i + 2 < input.len()
                    && let proc_macro2::TokenTree::Punct(punct) = &input[i + 1]
                    && punct.to_string() == "~"
                    && let proc_macro2::TokenTree::Ident(var) = &input[i + 2]
                    && var.to_string() == "N"
                {
                    let combined = format!("{}{}", ident, n_value);
                    let new_ident = proc_macro2::Ident::new(&combined, ident.span());
                    token_stream.append(new_ident);
                    i += 3; // Skip the ident, ~, and N
                } else if ident.to_string() == "N" {
                    // Standalone N replacement
                    let lit = syn::LitInt::new(&n_value.to_string(), ident.span());
                    token_stream.append(lit.token());
                    i += 1;
                } else {
                    token_stream.append(input[i].clone());
                    i += 1;
                }
            }
            _ => {
                token_stream.append(input[i].clone());
                i += 1
            }
        };
    }

    return token_stream;
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let user_macro = parse_macro_input!(input as SeqMacroInput);
    // println!("{:#?}", user_macro);
    let s: u8 = user_macro.range_start.base10_parse().unwrap();
    let e: u8 = user_macro.range_end.base10_parse().unwrap();
    // println!("{:?}", s);
    // let res = find_and_replace_N(user_macro.body, 4);
    // println!("here is the body: {:#?}", user_macro.body);
    let mut token_stream = TokenStream2::new();

    for i in s..e {
        let processed_body = find_and_replace_n(user_macro.body.clone(), i);
        // println!("Iteration {}: {}", i, processed_body);
        token_stream.extend(processed_body);
    }
    // println!("here is the output: {:?}", token_stream);
    // println!("here is the body: {:#?}", user_macro.body);
    token_stream.into()
}

// seq!(N in 0..4 {
//     compile_error!(concat!("error number ", stringify!(N)));
// });
