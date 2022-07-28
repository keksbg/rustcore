#![feature(extend_one)]
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{
    parse::Parser, parse_macro_input, AttributeArgs, Field, Fields, ItemStruct, Lit, Meta,
    NestedMeta, DeriveInput,
};

#[proc_macro_attribute]
/// ONLY MEANT FOR INTERNAL USE!
///
/// Creates registers. Syntax: `#[make_registers(type, length)]`
/// where `type` is a primitive type (like `u32`). `length`
/// is an integer representing the amount of registers to be created.
///
/// The function creates variables named `x$i` where `$i` is [0:length].
pub fn make_registers(args: TokenStream, body: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let pbody = parse_macro_input!(body as ItemStruct);

    assert!(args.len() == 2);

    // this is all just for extracting the arguments from the macro
    // help me
    let mut iter = args.into_iter();
    let _prim_type = match iter.next().unwrap() {
        NestedMeta::Meta(x) => match x {
            Meta::Path(y) => y,
            _ => panic!("wrong type"),
        },
        _ => panic!("wrong type"),
    };

    let prim_type = &_prim_type.segments.last().unwrap().ident;

    let len: usize = match iter.next().unwrap() {
        NestedMeta::Lit(x) => match x {
            Lit::Int(y) => y.base10_parse().unwrap(),
            _ => panic!("wrong type"),
        },
        _ => panic!("wrong type"),
    };

    let attrs = pbody.attrs;
    let vis = pbody.vis;
    let ident = pbody.ident;
    let generics = pbody.generics;
    let mut punct = match pbody.fields {
        Fields::Named(x) => x.named,
        _ => panic!("wrong type struct"),
    };

    let parser = Field::parse_named;
    for i in 0..len {
        let to_parse = format!("pub x{i}: {prim_type}").parse().unwrap();
        let parsed = parser.parse2(to_parse).unwrap();
        punct.push(parsed);
    }

    // re-create the input struct
    let out = quote! {
        #(#attrs)*
        #vis struct #ident #generics {
            #punct
        }
    };

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
