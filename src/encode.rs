use std::borrow::Cow;

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

pub fn rfc2047_encode(mut data: &str) -> Cow<str> {
    if data.is_ascii() {
        return Cow::Borrowed(data);
    }

    let mut result = String::with_capacity(data.len() * 2);
    let mut previous_token_was_encoded = false;

    let is_non_whitespace = |c: char| !c.is_ascii_whitespace();
    let is_whitespace = |c: char| c.is_ascii_whitespace();

    while !data.is_empty() {
        let word_begin = data.find(is_non_whitespace).unwrap_or(data.len());
        let word_end = word_begin
            + data[word_begin..]
                .find(is_whitespace)
                .unwrap_or(data.len() - word_begin);

        let word = &data[word_begin..word_end];
        if word.is_ascii() {
            let word_with_ws_prefix = &data[..word_end];
            result.push_str(word_with_ws_prefix);
            previous_token_was_encoded = false;
        } else {
            if previous_token_was_encoded {
                result.push(' ');
                let word_with_ws_prefix = &data[..word_end];
                let encoded_word = rfc2047_encode_word(word_with_ws_prefix);
                result.push_str(&encoded_word);
            } else {
                let prefix = &data[..word_begin];
                result.push_str(prefix);
                let encoded_word = rfc2047_encode_word(word);
                result.push_str(&encoded_word);
            }
            previous_token_was_encoded = true;
        }

        data = &data[word_end..];
    }

    Cow::Owned(result)
}

const MAX_LEN_ENCODED_WORD: usize = 75; // as per rfc 2047
const LEN_ENCODED_WORD_PREFIX: usize = 10; // "=?utf-8?q?"
const LEN_ENCODED_WORD_SUFFIX: usize = 2; // "?="
const LEN_ENCODED_WORD_BUFFER: usize = 4 * 3; // max. four bytes per encoded char
const MAX_LEN_ENCODED_DATA: usize = MAX_LEN_ENCODED_WORD
    - LEN_ENCODED_WORD_PREFIX
    - LEN_ENCODED_WORD_SUFFIX
    - LEN_ENCODED_WORD_BUFFER;

fn rfc2047_encode_word(word: &str) -> String {
    let mut charbuf = [0; 4];
    let mut result = String::with_capacity(word.len() + 15);
    result.push_str("=?utf-8?q?");

    let mut at = 0;
    for b in word.chars() {
        if at >= MAX_LEN_ENCODED_DATA {
            result.push_str("?= =?utf-8?q?");
            at = 0;
        }

        if b == ' ' {
            result.push('_');
            at += 1;
        } else if b.is_ascii() && b != '_' && b != '=' {
            result.push(b);
            at += 1;
        } else {
            for x in b.encode_utf8(&mut charbuf).as_bytes() {
                result.push('=');
                result.push(HEX_CHARS[(x >> 4) as usize]);
                result.push(HEX_CHARS[(x & 0xf) as usize]);
                at += 3;
            }
        }
    }

    result.push_str("?=");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_ascii() {
        assert_eq!("foo \n bar", rfc2047_encode("foo \n bar"));
    }

    #[test]
    fn just_whitespace() {
        assert_eq!("   ", rfc2047_encode("   "));
    }

    #[test]
    fn single_char() {
        assert_eq!("=?utf-8?q?=C3=A4?=", rfc2047_encode("ä"));
    }

    #[test]
    fn single_char_multi() {
        assert_eq!("=?utf-8?q?=F0=9F=9A=80?=", rfc2047_encode("🚀"));
    }

    #[test]
    fn encoded_word() {
        assert_eq!("=?utf-8?q?foo=C3=A1bar?=", rfc2047_encode("fooábar"));
    }

    #[test]
    fn encoded_at_start() {
        assert_eq!("=?utf-8?q?=C3=A4?=  bar", rfc2047_encode("ä  bar"));
    }

    #[test]
    fn encoded_at_end() {
        assert_eq!("Foo  =?utf-8?q?=C3=A4?=", rfc2047_encode("Foo  ä"));
    }

    #[test]
    fn encoded_in_middle() {
        assert_eq!(
            "Foo  =?utf-8?q?=C3=A4?=  bar",
            rfc2047_encode("Foo  ä  bar")
        );
    }

    #[test]
    fn encoded_adjacent_in_middle() {
        assert_eq!(
            "Foo  =?utf-8?q?=C3=A4?= =?utf-8?q?_=C3=A4?=  bar",
            rfc2047_encode("Foo  ä ä  bar")
        );
    }

    #[test]
    fn encoded_adjacent_with_whitespace() {
        assert_eq!(
            "=?utf-8?q?=C3=A4?= =?utf-8?q?__=C3=BC?=",
            rfc2047_encode("ä  ü")
        );
    }

    #[test]
    fn encoded_trailing_whitespace() {
        assert_eq!("=?utf-8?q?=C3=A4?=   ", rfc2047_encode("ä   "));
    }

    #[test]
    fn encoded_leading_whitespace() {
        assert_eq!("   =?utf-8?q?=C3=A4?=", rfc2047_encode("   ä"));
    }

    #[test]
    fn encoded_surrounding_whitespace() {
        assert_eq!("   =?utf-8?q?=C3=A4?=   ", rfc2047_encode("   ä   "));
    }

    #[test]
    fn encoded_long() {
        assert_eq!(
            "=?utf-8?q?=C3=A4012345678012345678012345678012345678012345678?= =?utf-8?q?0123456789999990123456789?=",
            rfc2047_encode("ä0123456780123456780123456780123456780123456780123456789999990123456789")
        );
    }
}
