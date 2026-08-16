# base64

A small, dependency-free Rust library for Base64 encoding and decoding, implementing the standard and URL-safe alphabets from [RFC 4648](https://datatracker.ietf.org/doc/html/rfc4648).

## Features

- Standard alphabet (`+`, `/`) with or without `=` padding
- URL-safe alphabet (`-`, `_`), unpadded
- Strict decoding that validates length, padding position, alphabet consistency, and rejects invalid bytes
- Encoding is available on any type that implements `AsRef<[u8]>` (e.g. `&str`, `String`, `Vec<u8>`, `&[u8]`) via the `Base64Encoder` trait

## Installation

This crate isn't published, so add it as a path or git dependency in `Cargo.toml`:

```toml
[dependencies]
base64 = { path = "../base64" }
```

## Usage

### Encoding

Bring the `Base64Encoder` trait into scope to get `encode_base64`, `encode_base64_url`, and `encode_base64_unpadded` on any `AsRef<[u8]>` type.

```rust
use base64::Base64Encoder;

fn main() {
    // Standard alphabet, padded (default)
    assert_eq!("Zm9vYmFy", "foobar".encode_base64());
    assert_eq!("Zm9vYmE=", "fooba".encode_base64());

    // Standard alphabet, no padding
    assert_eq!("Zm9vYmE", "fooba".encode_base64_unpadded());

    // URL-safe alphabet ('-' and '_'), unpadded
    assert_eq!("_A", [0xFCu8].encode_base64_url());

    // Works on any AsRef<[u8]>
    let bytes: Vec<u8> = vec![0x00, 0xFF];
    println!("{}", bytes.encode_base64());
}
```

### Decoding

`decode_base64` accepts anything that implements `AsRef<[u8]>` and returns a `Result<Vec<u8>, DecodeError>`. Both standard and URL-safe alphabets are recognized automatically (but not mixed within the same input), and padding is optional.

```rust
use base64::decode_base64;

fn main() {
    // Standard, padded
    assert_eq!(decode_base64("Zm9vYmFy").unwrap(), b"foobar");

    // Standard, unpadded
    assert_eq!(decode_base64("Zm9vYmE").unwrap(), b"fooba");

    // URL-safe
    assert_eq!(decode_base64("_A").unwrap(), vec![0xFC]);

    // Invalid input produces a descriptive error
    match decode_base64("AA!A") {
        Ok(bytes) => println!("decoded {} bytes", bytes.len()),
        Err(e) => println!("failed to decode: {:?}", e),
    }
}
```

### Round-trip example

```rust
use base64::{Base64Encoder, decode_base64};

fn main() {
    let original = "the quick brown fox";
    let encoded = original.encode_base64();
    let decoded = decode_base64(&encoded).unwrap();

    assert_eq!(original.as_bytes(), decoded.as_slice());
}
```

## Error handling

`decode_base64` returns `DecodeError` on invalid input:

| Variant | Meaning |
| --- | --- |
| `InvalidByte { index, byte }` | A byte outside both the standard and URL-safe alphabets was found. |
| `InvalidLength { length }` | The unpadded input length is congruent to 1 mod 4, which is not a valid Base64 length. |
| `InvalidPadding` | A `=` character is present but the total input length isn't a multiple of 4. |
| `InvalidPaddingPosition { index }` | A `=` character appears somewhere other than the end of the input. |
| `MixedAlphabet { index }` | The input mixes standard (`+`/`/`) and URL-safe (`-`/`_`) characters. |

```rust
use base64::{decode_base64, DecodeError};

let err = decode_base64("AA!A").unwrap_err();
assert_eq!(err, DecodeError::InvalidByte { index: 2, byte: b'!' });
```

## Running tests

```sh
cargo test
```
