# RFC 2047 Encoder

[![Build Status](https://travis-ci.org/Valodim/rust-rfc2047.svg?branch=master)](https://travis-ci.org/Valodim/rust-rfc2047)
![Crates.io](https://img.shields.io/crates/v/rfc2047)
![Crates.io](https://img.shields.io/crates/l/rfc2047)

Offers an encoder for RFC 2047 encoded words.

``` rust
use rfc2047::rfc2047_encode;

#[test]
fn test_encode_rfc2047() {
    assert_eq!(
        "Foo  =?utf-8?q?a=C3=A4b?= =?utf-8?q?_=C3=A4?=  bar",
        rfc2047_encode("Foo  aäb ä  bar"),
    );
}
```

Words are encoded or not encoded as a whole. Only quoted-printable encoding
is used.
