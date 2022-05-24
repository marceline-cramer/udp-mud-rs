use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

// with help from: https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs

#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let data = &input.data;

    let encode_members = match data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() =>
                        ::protocol::Encode::encode(&self.#name, writer)?;
                    }
                });
                quote! { #(#recurse)* }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl ::protocol::Encode for #name {
            fn encode(&self, writer: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
                #encode_members
                Ok(())
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(Decode)]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let data = &input.data;

    let decode_members = match data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() =>
                        #name: ::protocol::Decode::decode(reader)?,
                    }
                });
                quote! { #(#recurse)* }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        impl ::protocol::Decode for #name {
            fn decode(reader: &mut impl ::std::io::Read) -> ::std::io::Result<Self> {
                Ok(Self {
                    #decode_members
                })
            }
        }
    };

    expanded.into()
}
