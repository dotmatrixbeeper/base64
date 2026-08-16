const STD_CHARS: [char; 64] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 
                                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                                '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/'];

const URL_CHARS: [char; 64] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 
                                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                                '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '_'];

struct Config {
    alphabet: &'static [char; 64],
    pad: bool,
}

pub trait Base64Encoder {
    fn encode_base64(&self) -> String;
    fn encode_base64_url(&self) -> String;
    fn encode_base64_unpadded(&self) -> String;
}

impl<T: AsRef<[u8]> + ?Sized> Base64Encoder for T {
    fn encode_base64(&self) -> String {
        encode(self.as_ref(), Config { alphabet: &STD_CHARS, pad: true } )
    }

    fn encode_base64_url(&self) -> String {
        encode(self.as_ref(), Config { alphabet: &URL_CHARS, pad: false } )
    }

    fn encode_base64_unpadded(&self) -> String {
        encode(self.as_ref(), Config { alphabet: &STD_CHARS, pad: false } )
    }
}

fn encode(input: &[u8], config: Config) -> String {
    let mut encoded_result = String::new();
    for bytes in input.chunks(3) {
        if bytes.len() == 1 {
            encoded_result.push(config.alphabet[(bytes[0] >> 2) as usize]);
            encoded_result.push(config.alphabet[((bytes[0] & 0b0000_0011) << 4) as usize]);
            if config.pad {
                encoded_result.push_str("==");
            }
        } else if bytes.len() == 2 {
            encoded_result.push(config.alphabet[(bytes[0] >> 2) as usize]);
            encoded_result.push(config.alphabet[(((bytes[0] & 0b0000_0011) << 4) | (bytes[1] >> 4)) as usize]);
            encoded_result.push(config.alphabet[((bytes[1] & 0b0000_1111) << 2) as usize]);
            if config.pad {
                encoded_result.push_str("=");
            }
        } else {
            encoded_result.push(config.alphabet[(bytes[0] >> 2) as usize]);
            encoded_result.push(config.alphabet[(((bytes[0] & 0b0000_0011) << 4) | (bytes[1] >> 4)) as usize]);
            encoded_result.push(config.alphabet[(((bytes[1] & 0b0000_1111) << 2) | (bytes[2] >> 6)) as usize]);
            encoded_result.push(config.alphabet[(bytes[2] & 0b0011_1111) as usize]);
        }
    }
    return encoded_result;
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum DecodeError {
    InvalidByte { index: usize, byte: u8 },
    InvalidLength { length: usize },
    InvalidPadding,
    InvalidPaddingPosition { index: usize },
    MixedAlphabet { index: usize }
}

pub fn decode_base64(encoded: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    let encoded = encoded.as_ref();
    let encoded_len = encoded.len();

    let pad_len = match encoded {
        [.., b'=', b'='] => 2,
        [.., b'='] => 1,
        _ => 0
    };

    let data_len = encoded_len - pad_len;

    if let Some(index) = encoded[..data_len].iter().position(|&b| b == b'=') {
        return Err(DecodeError::InvalidPaddingPosition { index });
    }

    if data_len % 4 == 1 {
        return Err(DecodeError::InvalidLength { length: encoded_len });
    } else if pad_len > 0 && encoded_len % 4 != 0 {
        return Err(DecodeError::InvalidPadding);
    }

    let mut decoded = Vec::with_capacity(data_len * 3 / 4 + 2);

    let mut decoder_table = [0xFF; 256];
    let mut i = 0;
    while i < 64 {
        decoder_table[STD_CHARS[i] as usize] = i as u8;
        i += 1;
    }
    decoder_table[URL_CHARS[62] as usize] = 62;
    decoder_table[URL_CHARS[63] as usize] = 63;

    let data = &encoded[..data_len];
    let mut seen_std_special = false;
    let mut seen_url_special = false;

    for (i, group) in data.chunks(4).enumerate() {
        let base = i * 4;

        for (j, &byte) in group.iter().enumerate() {
            match byte {
                b'+' | b'/' => {
                    if seen_url_special {
                        return Err(DecodeError::MixedAlphabet { index: base + j });
                    }
                    seen_std_special = true;
                }
                b'-' | b'_' => {
                    if seen_std_special {
                        return Err(DecodeError::MixedAlphabet { index: base + j });
                    }
                    seen_url_special = true;
                }
                _ => {}
            }
        }

        if group.len() == 2 {
            let a = decoder_table[group[0] as usize];
            let b = decoder_table[group[1] as usize];

            if (a | b) == 0xFF {
                return Err(find_bad_byte(group, &decoder_table, base));
            }

            decoded.push((a << 2) | (b >> 4));
        } else if group.len() == 3 {
            let a = decoder_table[group[0] as usize];
            let b = decoder_table[group[1] as usize];
            let c = decoder_table[group[2] as usize];

            if (a | b | c) == 0xFF {
                return Err(find_bad_byte(group, &decoder_table, base));
            }

            decoded.push((a << 2) | (b >> 4));
            decoded.push((b << 4) | (c >> 2));
        } else {
            let a = decoder_table[group[0] as usize];
            let b = decoder_table[group[1] as usize];
            let c = decoder_table[group[2] as usize];
            let d = decoder_table[group[3] as usize];

            if (a | b | c | d) == 0xFF {
                return Err(find_bad_byte(group, &decoder_table, base));
            }

            decoded.push((a << 2) | (b >> 4));
            decoded.push((b << 4) | (c >> 2));
            decoded.push((c << 6) | d);
        }
    }
    return Ok(decoded);

}

#[cold]
#[inline(never)]
fn find_bad_byte(group: &[u8], decoder_table: &[u8; 256], base: usize) -> DecodeError {
    let mut index = 0;
    let mut error_byte = 0;
    for i in 0..group.len() {
        if decoder_table[group[i] as usize] == 0xFF {
            index = base + i;
            error_byte = group[i];
        }
    }
    return DecodeError::InvalidByte { index, byte: error_byte };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_positive_test() {
        let config = Config {
            alphabet: &STD_CHARS,
            pad: true
        };

        assert_eq!("YWI=", encode("ab".as_bytes(), config));
    }

    // ---- encode_base64 (standard, padded) — RFC 4648 test vectors ----

    #[test]
    fn encode_std_rfc4648_vectors() {
        assert_eq!("", "".encode_base64());
        assert_eq!("Zg==", "f".encode_base64());
        assert_eq!("Zm8=", "fo".encode_base64());
        assert_eq!("Zm9v", "foo".encode_base64());
        assert_eq!("Zm9vYg==", "foob".encode_base64());
        assert_eq!("Zm9vYmE=", "fooba".encode_base64());
        assert_eq!("Zm9vYmFy", "foobar".encode_base64());
    }

    #[test]
    fn encode_std_uses_plus_and_slash() {
        assert_eq!("+A==", [0xF8u8].encode_base64());
        assert_eq!("/w==", [0xFFu8].encode_base64());
        assert_eq!("////", [0xFFu8, 0xFF, 0xFF].encode_base64());
    }

    // ---- encode_base64_unpadded (standard alphabet, no padding) ----

    #[test]
    fn encode_unpadded_rfc4648_vectors() {
        assert_eq!("", "".encode_base64_unpadded());
        assert_eq!("Zg", "f".encode_base64_unpadded());
        assert_eq!("Zm8", "fo".encode_base64_unpadded());
        assert_eq!("Zm9v", "foo".encode_base64_unpadded());
        assert_eq!("Zm9vYg", "foob".encode_base64_unpadded());
        assert_eq!("Zm9vYmE", "fooba".encode_base64_unpadded());
        assert_eq!("Zm9vYmFy", "foobar".encode_base64_unpadded());
    }

    // ---- encode_base64_url (url-safe alphabet, no padding) ----

    #[test]
    fn encode_url_rfc4648_vectors() {
        assert_eq!("", "".encode_base64_url());
        assert_eq!("Zg", "f".encode_base64_url());
        assert_eq!("Zm8", "fo".encode_base64_url());
        assert_eq!("Zm9v", "foo".encode_base64_url());
        assert_eq!("Zm9vYg", "foob".encode_base64_url());
        assert_eq!("Zm9vYmE", "fooba".encode_base64_url());
        assert_eq!("Zm9vYmFy", "foobar".encode_base64_url());
    }

    #[test]
    fn encode_url_uses_dash_and_underscore() {
        assert_eq!("-A", [0xF8u8].encode_base64_url());
        assert_eq!("_w", [0xFFu8].encode_base64_url());
        assert_eq!("____", [0xFFu8, 0xFF, 0xFF].encode_base64_url());
    }

    #[test]
    fn encode_accepts_various_input_types() {
        let bytes: Vec<u8> = vec![b'a', b'b'];
        let owned = String::from("ab");
        assert_eq!("YWI=", "ab".encode_base64());
        assert_eq!("YWI=", owned.encode_base64());
        assert_eq!("YWI=", bytes.encode_base64());
        assert_eq!("YWI=", bytes.as_slice().encode_base64());
    }

    // ---- decode_base64: success cases ----

    #[test]
    fn decode_empty() {
        assert_eq!(Ok(vec![]), decode_base64(""));
    }

    #[test]
    fn decode_std_rfc4648_vectors_padded() {
        assert_eq!(Ok(b"f".to_vec()), decode_base64("Zg=="));
        assert_eq!(Ok(b"fo".to_vec()), decode_base64("Zm8="));
        assert_eq!(Ok(b"foo".to_vec()), decode_base64("Zm9v"));
        assert_eq!(Ok(b"foob".to_vec()), decode_base64("Zm9vYg=="));
        assert_eq!(Ok(b"fooba".to_vec()), decode_base64("Zm9vYmE="));
        assert_eq!(Ok(b"foobar".to_vec()), decode_base64("Zm9vYmFy"));
    }

    #[test]
    fn decode_std_rfc4648_vectors_unpadded() {
        assert_eq!(Ok(b"f".to_vec()), decode_base64("Zg"));
        assert_eq!(Ok(b"fo".to_vec()), decode_base64("Zm8"));
        assert_eq!(Ok(b"foob".to_vec()), decode_base64("Zm9vYg"));
        assert_eq!(Ok(b"fooba".to_vec()), decode_base64("Zm9vYmE"));
    }

    #[test]
    fn decode_url_alphabet() {
        assert_eq!(Ok(vec![0xF8]), decode_base64("-A"));
        assert_eq!(Ok(vec![0xFF]), decode_base64("_w"));
        assert_eq!(Ok(vec![0xFF, 0xFF, 0xFF]), decode_base64("____"));
    }

    #[test]
    fn decode_std_alphabet_boundary_values() {
        assert_eq!(Ok(vec![0]), decode_base64("AA=="));
        assert_eq!(Ok(vec![0, 0]), decode_base64("AAA="));
        assert_eq!(Ok(vec![255]), decode_base64("//=="));
        assert_eq!(Ok(vec![255, 255, 255]), decode_base64("////"));
    }

    #[test]
    fn decode_roundtrip_all_lengths_and_all_byte_values() {
        // every byte value, at every remainder-length (0,1,2 mod 3), through every encoder
        let data: Vec<u8> = (0..=255).collect();
        for len in 0..data.len() {
            let slice = &data[..len];

            let std_encoded = slice.encode_base64();
            assert_eq!(Ok(slice.to_vec()), decode_base64(&std_encoded), "std roundtrip failed at len {len}");

            let unpadded_encoded = slice.encode_base64_unpadded();
            assert_eq!(Ok(slice.to_vec()), decode_base64(&unpadded_encoded), "unpadded roundtrip failed at len {len}");

            let url_encoded = slice.encode_base64_url();
            assert_eq!(Ok(slice.to_vec()), decode_base64(&url_encoded), "url roundtrip failed at len {len}");
        }
    }

    // ---- decode_base64: InvalidLength ----

    #[test]
    fn decode_rejects_length_congruent_to_one_mod_four() {
        assert_eq!(Err(DecodeError::InvalidLength { length: 1 }), decode_base64("A"));
        assert_eq!(Err(DecodeError::InvalidLength { length: 5 }), decode_base64("AAAAA"));
        assert_eq!(Err(DecodeError::InvalidLength { length: 9 }), decode_base64("AAAAAAAAA"));
    }

    // ---- decode_base64: InvalidPadding ----

    #[test]
    fn decode_rejects_padding_that_leaves_bad_total_length() {
        // single '=' but total length not a multiple of 4
        assert_eq!(Err(DecodeError::InvalidPadding), decode_base64("AA="));
        // double '==' but total length not a multiple of 4
        assert_eq!(Err(DecodeError::InvalidPadding), decode_base64("AAAA=="));
    }

    // ---- decode_base64: InvalidPaddingPosition ----

    #[test]
    fn decode_rejects_padding_in_the_middle() {
        assert_eq!(Err(DecodeError::InvalidPaddingPosition { index: 1 }), decode_base64("A=AA"));
    }

    #[test]
    fn decode_rejects_more_than_two_padding_characters() {
        assert_eq!(Err(DecodeError::InvalidPaddingPosition { index: 1 }), decode_base64("A==="));
    }

    // ---- decode_base64: InvalidByte ----

    #[test]
    fn decode_rejects_byte_outside_alphabet() {
        assert_eq!(Err(DecodeError::InvalidByte { index: 2, byte: b'!' }), decode_base64("AA!A"));
    }

    #[test]
    fn decode_rejects_byte_outside_alphabet_in_later_group() {
        assert_eq!(Err(DecodeError::InvalidByte { index: 4, byte: b'!' }), decode_base64("AAAA!AAA"));
    }

    #[test]
    fn decode_rejects_byte_outside_alphabet_in_unpadded_tail() {
        assert_eq!(Err(DecodeError::InvalidByte { index: 4, byte: b'!' }), decode_base64("AAAA!A"));
    }

    // ---- decode_base64: MixedAlphabet ----

    #[test]
    fn decode_rejects_mixed_alphabet() {
        // '+' (std) in the first group, '-' (url) in the second group
        let result = decode_base64("AA+AAA-A");
        assert_eq!(Err(DecodeError::MixedAlphabet { index: 6 }), result);
    }

    #[test]
    fn decode_rejects_mixed_alphabet_within_same_group() {
        assert_eq!(Err(DecodeError::MixedAlphabet { index: 3 }), decode_base64("A+A_"));
    }

    #[test]
    fn decode_rejects_mixed_alphabet_url_then_std() {
        assert_eq!(Err(DecodeError::MixedAlphabet { index: 3 }), decode_base64("A-A+"));
    }

    #[test]
    fn decode_rejects_mixed_alphabet_in_unpadded_tail() {
        assert_eq!(Err(DecodeError::MixedAlphabet { index: 6 }), decode_base64("AAAA-A+A"));
    }

    #[test]
    fn decode_accepts_pure_std_and_pure_url_without_false_positive_clash() {
        assert!(decode_base64("AA+/AA+/").is_ok());
        assert!(decode_base64("AA-_AA-_").is_ok());
    }
}