#![feature(extend_one)]
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::Parser, parse_macro_input, AttributeArgs, Field, Fields, ItemStruct, Lit, Meta,
    NestedMeta, DeriveInput, spanned::Spanned,
};

#[proc_macro_attribute]
/// ONLY MEANT FOR INTERNAL USE!
///
/// Creates registers. Syntax: `#[make_registers(type, length)]`
/// where `type` is a type (like `u32`). `length` is an integer
/// representing the amount of registers to be created.
///
/// This only works on structs with named fields.
///
/// The function creates variables named `x$i` where `$i` is 0..length.
pub fn make_registers(args: TokenStream, body: TokenStream) -> TokenStream {
    let nargs = parse_macro_input!(args as AttributeArgs);
    let pbody = parse_macro_input!(body as ItemStruct);

    assert!(nargs.len() == 2);

    let mut iter = nargs.into_iter();
    let next = iter.next().unwrap();
    let span = next.span();

    let mut out = proc_macro2::TokenStream::new();

    let _prim_type = match next {
        NestedMeta::Meta(x) => match x {
            Meta::Path(y) => y,
            _ => {
                quote_spanned!(span=> compile_error!("expected type")).to_tokens(&mut out);
                return out.into();
            }
        },
        _ => {
            quote_spanned!(span=> compile_error!("expected type")).to_tokens(&mut out);
            return out.into();
        }
    };

    let prim_type = &_prim_type.segments.last().unwrap().ident;

    let next = iter.next().unwrap();
    let span = next.span();

    let len: usize = match next {
        NestedMeta::Lit(x) => match x {
            Lit::Int(y) => y.base10_parse().unwrap(),
            _ => {
                quote_spanned!(span=> compile_error!("expected integer")).to_tokens(&mut out);
                return out.into();
            },
        },
        _ => {
            quote_spanned!(span=> compile_error!("expected integer")).to_tokens(&mut out);
            return out.into();
        },
    };

    let span = pbody.span();
    let attrs = pbody.attrs;
    let vis = pbody.vis;
    let ident = pbody.ident;
    let generics = pbody.generics;
    let fs = pbody.fields;
    let mut punct = match fs {
        Fields::Named(x) => x.named,
        _ => {
            quote_spanned!(span=> compile_error!("expected struct with named fields")).to_tokens(&mut out);
            return out.into();
        },
    };

    let parser = Field::parse_named;
    for i in 0..len {
        let to_parse = format!("pub x{i}: {prim_type}").parse().unwrap();
        let parsed = parser.parse2(to_parse).unwrap();
        punct.push(parsed);
    }

    // re-create the input struct
    quote! {
        #(#attrs)*
        #vis struct #ident #generics {
            #punct
        }
    }.to_tokens(&mut out);

    out.into()
}

#[proc_macro_derive(Instruction)]
pub fn derive_instruction(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    quote! {
        impl crate::base::Instruction for #name {}
    }.into()
}
