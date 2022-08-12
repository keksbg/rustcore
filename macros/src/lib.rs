#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::Parser, parse_macro_input, spanned::Spanned, AttributeArgs, DeriveInput, Field, Fields,
    ItemStruct, Lit, Meta, NestedMeta, MetaNameValue,
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

#[proc_macro_derive(Instruction, attributes(bits))]
pub fn derive_instruction(input: TokenStream) -> TokenStream {
    let mut out = proc_macro2::TokenStream::new();

    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
    let name = input.ident;
    // get the fields as a Punctuated<T, P>, then as an iterator over T
    let fields = match input.data {
        syn::Data::Struct(s) => {
            match s.fields {
                syn::Fields::Named(f) => f.named.into_iter(),
                _ => return quote_spanned!{span=>
                    compile_error!("expected a struct with named fields");
                }.into()
            }
        },
        _ => return quote_spanned!{span=>
            compile_error!("expected a struct");
        }.into()
    };

    // get the bits from the attribute
    let bits: Vec<syn::Lit> = fields.clone().map(|mut f| {
        match f.attrs
               .pop()
               .unwrap()
               .parse_meta()
               .unwrap()
            {
                Meta::NameValue(nv) => nv.lit,
                _ => panic!("expected `name = value` type field attribute"),
            }
    }).collect();

    let sum: u8 = bits.clone().into_iter().map(|lit| {
        return match lit {
            syn::Lit::Int(i) => i.base10_parse::<u8>().unwrap(),
            _ => panic!("field attribute expects integer type for value"),
        }
    }).sum();

    if sum != 32 {
        let fmt = format!("the sum of bits in the struct ({} bits) does not equal the size of a `u32` (32 bits)", sum);
        quote_spanned! {span=>
            compile_error!(#fmt);
        }.to_tokens(&mut out);
    }

    // get the field name (ident)
    let ident: Vec<syn::Ident> = fields.clone().map(|f| f.ident.unwrap()).collect();
    // get the field type
    let ty: Vec<syn::Type> = fields.clone().map(|f| f.ty).collect();

    quote! {
        impl crate::base::Instruction for #name {
            fn from_u32(input: u32) -> Self {
                let mut _i = 0;
                Self {
                    // honestly i don't know how the fuck it was able to repeat correctly here
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
            fn to_u32(&self) -> u32 {
                let mut _i = 0;
                let mut out = 0;
                #(
                    for i in 0..#bits {
                        // considering `i` will always be less/eq to `_i`, instead of
                        // shifting right by `i` and then by `_i` we can shift
                        // to the left by their difference
                        out |= ((self.#ident & (1 << i)) as u32) << (_i - i);
                        _i += 1;
                    }
                )*
                out
            }
        }
    }.to_tokens(&mut out);

    out.into()
}
