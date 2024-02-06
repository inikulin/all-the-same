//! If you ever had code that looks like this:
//!
//! ```
//! use std::io;
//! use std::pin::Pin;
//! use std::task::{Context, Poll};
//! use tokio::io::AsyncWrite;
//! use tokio::net::{TcpStream, UnixStream};
//!
//! enum Stream {
//!     Tcp(TcpStream),
//!     Unix(UnixStream),
//!     Custom(Box<dyn AsyncWrite + Unpin + 'static>),
//! }
//!
//! impl AsyncWrite for Stream {
//!     fn poll_write(
//!         self: Pin<&mut Self>,
//!         cx: &mut Context<'_>,
//!         buf: &[u8],
//!     ) -> Poll<Result<usize, io::Error>> {
//!         match self.get_mut() {
//!             Stream::Tcp(s) => Pin::new(s).poll_write(cx, buf),
//!             Stream::Unix(s) => Pin::new(s).poll_write(cx, buf),
//!             Stream::Custom(s) => Pin::new(s).poll_write(cx, buf),
//!         }
//!     }
//!
//!     fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
//!         match self.get_mut() {
//!             Stream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
//!             Stream::Unix(s) => Pin::new(s).poll_shutdown(cx),
//!             Stream::Custom(s) => Pin::new(s).poll_shutdown(cx),
//!         }
//!     }
//!
//!     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
//!         match self.get_mut() {
//!             Stream::Tcp(s) => Pin::new(s).poll_flush(cx),
//!             Stream::Unix(s) => Pin::new(s).poll_flush(cx),
//!             Stream::Custom(s) => Pin::new(s).poll_flush(cx),
//!         }
//!     }
//! }
//! ```
//!
//! with the help of the macro you can now replace it with:
//! ```
//! use std::io;
//! use std::pin::Pin;
//! use std::task::{Context, Poll};
//! use tokio::io::AsyncWrite;
//! use tokio::net::{TcpStream, UnixStream};
//! use all_the_same::all_the_same;
//!
//! enum Stream {
//!     Tcp(TcpStream),
//!     Unix(UnixStream),
//!     Custom(Box<dyn AsyncWrite + Unpin + 'static>),
//! }
//!
//! impl AsyncWrite for Stream {
//!     fn poll_write(
//!         self: Pin<&mut Self>,
//!         cx: &mut Context<'_>,
//!         buf: &[u8],
//!     ) -> Poll<Result<usize, io::Error>> {
//!         all_the_same!(match self.get_mut() {
//!             Stream::[Tcp, Unix, Custom](s) => Pin::new(s).poll_write(cx, buf)
//!         })
//!     }
//!
//!     fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
//!         all_the_same!(match self.get_mut() {
//!             Stream::[Tcp, Unix, Custom](s) => Pin::new(s).poll_shutdown(cx)
//!         })
//!     }
//!
//!     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
//!         all_the_same!(match self.get_mut() {
//!             Stream::[Tcp, Unix, Custom](s) => Pin::new(s).poll_flush(cx)
//!         })
//!     }
//! }
//! ```
//!
//! # Feature gated enum variants, etc.
//!
//! Btw, you can add attributes that will be applied to the match arms, to deal with feature-gated
//! enum variants:
//!
//! ```
//! use all_the_same::all_the_same;
//!
//! enum Variants {
//!     Foo(String),
//!     
//!     #[cfg(test)]
//!     Bar(String)
//! }
//!
//! impl Variants {
//!     pub fn value(&self) -> &str {
//!         all_the_same!(match self {
//!             Variants::[Foo, #[cfg(test)]Bar](v) => v
//!         })
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{braced, bracketed, parenthesized, parse_macro_input, Attribute, Expr, Ident, Token};

struct Variant {
    attrs: Vec<Attribute>,
    name: Ident,
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Variant {
            attrs: input.call(Attribute::parse_outer)?,
            name: input.parse()?,
        })
    }
}

struct Args {
    expr: Expr,
    enum_name: Ident,
    variants: Punctuated<Variant, Comma>,
    inner_name: Ident,
    arm_expr: Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let match_body_content;

        Ok(Args {
            expr: {
                input.parse::<Token!(match)>()?;

                Expr::parse_without_eager_brace(input)?
            },
            enum_name: {
                braced!(match_body_content in input);

                match_body_content.parse()?
            },
            variants: {
                match_body_content.parse::<Token!(::)>()?;

                let variants_list_content;

                bracketed!(variants_list_content in match_body_content);

                variants_list_content.parse_terminated(Variant::parse)?
            },
            inner_name: {
                let variant_payload_content;

                parenthesized!(variant_payload_content in match_body_content);

                variant_payload_content.parse()?
            },
            arm_expr: {
                match_body_content.parse::<Token!(=>)>()?;

                match_body_content.parse()?
            },
        })
    }
}

/// The macro itself.
#[proc_macro]
pub fn all_the_same(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as Args);

    let expr = &args.expr;
    let enum_name = &args.enum_name;
    let inner_name = &args.inner_name;
    let arm_expr = &args.arm_expr;

    let arms = args.variants.iter().map(|variant| {
        let name = &variant.name;
        let attrs = &variant.attrs;

        quote! {
            #(#attrs)*
            #enum_name::#name(#inner_name) => #arm_expr
        }
    });

    quote! {
        match #expr {
            #(#arms),*
        }
    }
    .into()
}
