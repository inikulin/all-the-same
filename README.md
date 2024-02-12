# All the same!

[![crates.io](https://img.shields.io/crates/v/all-the-same.svg)](https://crates.io/crates/all-the-same)
[![docs.rs](https://docs.rs/all-the-same/badge.svg)](https://docs.rs/all-the-same)

If you ever had code that looks like this:

```rust
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncWrite;
use tokio::net::{TcpStream, UnixStream};

enum Stream {
    Tcp(TcpStream),
    Unix(UnixStream),
    Custom(Box<dyn AsyncWrite + Unpin + 'static>),
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            Stream::Tcp(s) => Pin::new(s).poll_write(cx, buf),
            Stream::Unix(s) => Pin::new(s).poll_write(cx, buf),
            Stream::Custom(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Stream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            Stream::Unix(s) => Pin::new(s).poll_shutdown(cx),
            Stream::Custom(s) => Pin::new(s).poll_shutdown(cx),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Stream::Tcp(s) => Pin::new(s).poll_flush(cx),
            Stream::Unix(s) => Pin::new(s).poll_flush(cx),
            Stream::Custom(s) => Pin::new(s).poll_flush(cx),
        }
    }
}
```

with the help of the macro you can now replace it with:

```rust
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncWrite;
use tokio::net::{TcpStream, UnixStream};
use all_the_same::all_the_same;

enum Stream {
    Tcp(TcpStream),
    Unix(UnixStream),
    Custom(Box<dyn AsyncWrite + Unpin + 'static>),
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        all_the_same!(match self.get_mut() {
            Stream::[Tcp, Unix, Custom](s) => Pin::new(s).poll_write(cx, buf)
        })
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), io::Error>> {
        all_the_same!(match self.get_mut() {
            Stream::[Tcp, Unix, Custom](s) => Pin::new(s).poll_shutdown(cx)
        })
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        all_the_same!(match self.get_mut() {
            Stream::[Tcp, Unix, Custom](s) => Pin::new(s).poll_flush(cx)
        })
    }
}
```

# Feature gated enum variants, etc.

Btw, you can add attributes that will be applied to the match arms, to deal with feature-gated
enum variants:

```rust
use all_the_same::all_the_same;

enum Variants {
    Foo(String),

    #[cfg(test)]
    Bar(String)
}

impl Variants {
    pub fn value(&self) -> &str {
        all_the_same!(match self {
            Variants::[Foo, #[cfg(test)]Bar](v) => v
        })
    }
}
```
