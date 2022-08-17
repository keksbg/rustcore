#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_macro_input,
    AttributeArgs, DeriveInput, Error, Field, Fields, ItemStruct, Lit, LitInt, Meta, NestedMeta,
    Result, Token,
};

macro_rules! bail {
    ( $msg:expr $(,)? ) => {
        return ::syn::Result::Err(::syn::Error::new(::proc_macro2::Span::mixed_site(), &$msg))
    };
    ( $msg:expr => $spanned:expr $(,)? ) => {
        return ::syn::Result::Err(::syn::Error::new_spanned(&$spanned, &$msg))
    };
}

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
    let args = parse_macro_input!(args as AttributeArgs);
    let body = parse_macro_input!(body as ItemStruct);

    make_registers_impl(args, body)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn make_registers_impl(args: AttributeArgs, body: ItemStruct) -> Result<TokenStream2> {
    if args.len() != 2 {
        let fmt = format!("expected 2 arguments, found {}", args.len());
        bail!(fmt);
    }

    let mut iter = args.into_iter();

    let _prim_type = match iter.next().unwrap() {
        NestedMeta::Meta(x) => match x {
            Meta::Path(y) => y,
            m => bail!("expected type" => m),
        },
        nm => bail!("expected type" => nm),
    };

    let prim_type = &_prim_type.segments.last().unwrap().ident;

    let next = iter.next().unwrap();

    let len: usize = match next {
        NestedMeta::Lit(x) => match x {
            Lit::Int(y) => y.base10_parse().unwrap(),
            lit => bail!("expected integer" => lit),
        },
        nm => bail!("expected integer" => nm),
    };

    let attrs = body.attrs;
    let vis = body.vis;
    let ident = body.ident;
    let generics = body.generics;
    let fs = body.fields;
    let mut punct = match fs {
        Fields::Named(x) => x.named,
        f => bail!("expected struct with named fields" => f),
    };

    let parser = Field::parse_named;
    for i in 0..len {
        let to_parse = format!("pub x{i}: {prim_type}").parse().unwrap();
        let parsed = parser.parse2(to_parse).unwrap();
        punct.push(parsed);
    }

    // re-create the input struct
    Result::Ok(quote! {
        #(#attrs)*
        #vis struct #ident #generics {
            #punct
        }
    })
}

#[proc_macro_derive(Instruction, attributes(bits))]
pub fn derive_instruction(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    instruction_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn instruction_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = input.clone().ident;
    // get the fields as a Punctuated<T, P>, then as an iterator over T
    let fields = match input.clone().data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(f) => f.named.into_iter(),
            st => bail!("expected a struct with named fields" => st),
        },
        _ => bail!("expected a struct"),
    };

    // get the bits from the attribute
    let bits: Vec<syn::Lit> = fields
        .clone()
        .map(|mut f| {
            Result::<_>::Ok({
                match f.attrs.pop().unwrap().parse_meta().unwrap() {
                    Meta::NameValue(nv) => nv.lit,
                    x => bail!("expected `name = value` type field attribute" => x),
                }
            })
        })
        .collect::<Result<_>>()?;

    let sum: u8 = bits
        .clone()
        .into_iter()
        .map(|lit| {
            syn::Result::<_>::Ok({
                match lit {
                    syn::Lit::Int(i) => i.base10_parse::<u8>().unwrap(),
                    lit => bail!("field attribute expects integer type for value" => lit),
                }
            })
        })
        .sum::<Result<_>>()?;

    if sum != 32 {
        let fmt = format!(
            "the sum of bits in the struct ({} bits) does not equal the size of a `u32` (32 bits)",
            sum
        );
        bail!(fmt => input);
    }

    // get the field name (ident)
    let ident: Vec<syn::Ident> = fields.clone().map(|f| f.ident.unwrap()).collect();
    // get the field type
    let ty: Vec<syn::Type> = fields.map(|f| f.ty).collect();

    syn::Result::Ok(quote! {
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
                        out |= ((self.#ident as u32 & (1 << i)) as u32) << (_i - i);
                        _i += 1;
                    }
                )*
                out
            }
        }
    })
}

struct GenOpcodesInput {
    punct: syn::punctuated::Punctuated<OpcodesField, Token![,]>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct OpcodesField {
    ident: syn::Ident,
    eq_token: Token![=],
    literal: LitInt,
    fat_arrow_token: Token![=>],
    instruction_type: syn::Ident,
}

impl Parse for OpcodesField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(OpcodesField {
            ident: input.parse()?,
            eq_token: input.parse()?,
            literal: input.parse()?,
            fat_arrow_token: input.parse()?,
            instruction_type: input.parse()?,
        })
    }
}

impl Parse for GenOpcodesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(GenOpcodesInput {
            punct: input.parse_terminated(OpcodesField::parse)?,
        })
    }
}

#[proc_macro]
pub fn gen_opcodes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GenOpcodesInput);

    gen_opcodes_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn gen_opcodes_impl(input: GenOpcodesInput) -> Result<TokenStream2> {
    let vec = input.punct.into_iter();
    let ident: Vec<syn::Ident> = vec.clone().map(|v| v.ident).collect();
    let lit: Vec<syn::LitInt> = vec.clone().map(|v| v.literal).collect();
    let inst_type: Vec<syn::Ident> = vec.clone().map(|v| v.instruction_type).collect();
    let inst: Vec<syn::Ident> = vec
        .map(|v| format_ident!("{}Instruction", v.instruction_type))
        .collect();
    Ok(quote! {
        #[repr(u8)]
        pub enum OpCodes {
            #(#ident = #lit),*
        }

        pub fn decode_instruction(input: u32) -> Result<Instructions, ()> {
            let opcode = (input & 0b0111_1111) as u8;
            match opcode {
                #(
                    #lit => Ok(
                        crate::base::Instructions::#inst_type(
                            crate::base::#inst::from_u32(input)
                        )
                    ),
                )*
                _ => Err(()),
            }
        }
    })
}
