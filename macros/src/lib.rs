#![feature(extend_one)]
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::Parser, parse_macro_input, spanned::Spanned, AttributeArgs, DeriveInput, Field, Fields,
    ItemStruct, Lit, Meta, NestedMeta,
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
            }
        },
        _ => {
            quote_spanned!(span=> compile_error!("expected integer")).to_tokens(&mut out);
            return out.into();
        }
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
            quote_spanned!(span=> compile_error!("expected struct with named fields"))
                .to_tokens(&mut out);
            return out.into();
        }
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
    }
    .to_tokens(&mut out);

    out.into()
}

// TODO: proper error handling
#[proc_macro_derive(Instruction, attributes(bits))]
pub fn derive_instruction(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    // get the fields as a Punctuated<T, P>, then as an iterator over T
    let fields = match input.data {
        syn::Data::Struct(s) => {
            match s.fields {
                syn::Fields::Named(f) => f.named.into_iter(),
                _ => panic!("nop")
            }
        },
        _ => panic!("yup")
    };

    // get the bits from the attribute
    let bits = fields.clone().map(|mut f| {
        match f.attrs
               .pop()
               .unwrap()
               .parse_meta()
               .unwrap()
            {
                Meta::NameValue(nv) => nv.lit,
                _ => panic!("oop"),
            }
    });
    // get the field name (ident)
    let ident = fields.clone().map(|f| f.ident.unwrap());
    // get the field type
    let ty = fields.clone().map(|f| f.ty);

    quote! {
        impl crate::base::Instruction for #name {
            fn from_u32(input: u32) -> Self {
                let mut _i = 0;
                Self {
                    // honestly i don't know how the fuck it was able to
                    #(
                        #ident: {
                            let mut _r = 0;
                            for i in 0..#bits {
                                // bit shift magic (grab the bit we need, then shift right
                                // the same amount so we get to the "beginning", then shift
                                // to the left `i` times (`_i - i`))
                                _r |= ((input & (1 << _i)) >> (_i - i)) as #ty;
                                _i += 1;
                            }
                            _r
                        }
                    ),*
                }
            }
        }
    }
    .into()
}
