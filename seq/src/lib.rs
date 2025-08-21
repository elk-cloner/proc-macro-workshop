use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, TokenStreamExt};
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

fn find_and_replace_N(input: TokenStream2, n_value: u8) -> TokenStream2 {
    let mut token_stream = TokenStream2::new();

    for token in input.clone() {
        match &token {
            proc_macro2::TokenTree::Group(group) => {
                let c = find_and_replace_N(group.stream(), n_value);
                let c =
                    proc_macro2::TokenTree::Group(proc_macro2::Group::new(group.delimiter(), c));
                token_stream.append(c);
            }
            proc_macro2::TokenTree::Ident(ident) if ident.to_string() == "N" => {
                let lit = syn::LitInt::new(&n_value.to_string(), ident.span());
                token_stream.append(lit.token());
            }
            _ => {
                token_stream.append(token);
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
    // println!("here is the output: {:#?}", res);
    let mut token_stream = TokenStream2::new();

    for i in s..e {
        let processed_body = find_and_replace_N(user_macro.body.clone(), i);
        println!("Iteration {}: {}", i, processed_body);
        token_stream.extend(processed_body);
    }
    println!("here is the output: {:?}", token_stream);
    // println!("here is the body: {:#?}", user_macro.body);
    token_stream.into()
}

// seq!(N in 0..4 {
//     compile_error!(concat!("error number ", stringify!(N)));
// });
