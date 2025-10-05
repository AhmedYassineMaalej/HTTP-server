use std::collections::{HashMap, hash_map::Iter};

use crate::request::{ParseError, ParserState, SEPARATOR};

const VALID_SYMBOLS: [u8; 15] = [
    b'!', b'#', b'$', b'%', b'&', b'\'', b'*', b'+', b'-', b'.', b'^', b'_', b'`', b'|', b'~',
];

#[derive(Debug, Default, PartialEq)]
pub struct Headers {
    inner: HashMap<String, String>,
}

impl Headers {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: &str, value: String) {
        let key = key.to_lowercase();

        self.inner
            .entry(key)
            .and_modify(|v| {
                v.push_str(", ");
                v.push_str(&value);
            })
            .or_insert(value);
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(&key.to_lowercase())
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_, String, String> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a String, &'a String);
    type IntoIter = Iter<'a, String, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

#[derive(Default)]
pub struct HeadersParser {
    headers: Headers,
    state: ParserState,
}

impl HeadersParser {
    #[must_use]
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
            state: ParserState::Init,
        }
    }

    /// Parse data inside `data` buffer and return amount of bytes consumed
    /// return `Ok(0)` if not data is consumed
    ///
    /// # Errors
    ///
    /// This function will return:
    /// - `Error::MissingFieldColumn` if `:` is missing from field line
    /// - `Error::InvalidFieldLineSpacing` if the spacing of field line does not conform to the
    ///   HTTP/1.1 standard
    /// - `Error::InvalidTokenCharacter` if it encounters an invalid header character
    pub fn parse(&mut self, data: &[u8]) -> Result<usize, ParseError> {
        self.state = ParserState::Init;
        let mut read = 0;

        while self.state == ParserState::Init {
            read += self.parse_header(&data[read..])?;
        }

        Ok(read)
    }

    /// Parse data inside `data` buffer and return amount of bytes consumed
    /// return `Ok(0)` if not data is consumed
    ///
    /// # Errors
    ///
    /// This function will return:
    /// - `Error::MissingFieldColumn` if `:` is missing from field line
    /// - `Error::InvalidFieldLineSpacing` if the spacing of field line does not conform to the
    ///   HTTP/1.1 standard
    /// - `Error::InvalidTokenCharacter` if it encounters an invalid header character
    fn parse_header(&mut self, data: &[u8]) -> Result<usize, ParseError> {
        let Some(idx) = data
            .windows(SEPARATOR.len())
            .position(|window| window == SEPARATOR)
        else {
            self.state = ParserState::Waiting;
            return Ok(0);
        };

        let line = &data[..idx];
        let read = idx + SEPARATOR.len();

        if idx == 0 {
            self.state = ParserState::Done;
            return Ok(read);
        }

        let Some(colon_idx) = line.iter().position(|b| b == &b':') else {
            return Err(ParseError::MissingFieldLineColumn);
        };

        let (key, value) = (&line[..colon_idx], &line[colon_idx + 1..]);

        if key.ends_with(b" ") {
            return Err(ParseError::InvalidFieldLineSpacing);
        }

        if !Self::valid_key(key) {
            return Err(ParseError::InvalidTokenCharacter);
        }

        let key = String::from_utf8_lossy(key.trim_ascii()).into_owned();
        let value = String::from_utf8_lossy(value.trim_ascii()).into_owned();

        self.headers.insert(&key, value);

        Ok(read)
    }

    #[must_use]
    pub fn inner_headers(self) -> Headers {
        self.headers
    }

    #[must_use]
    pub fn done(&self) -> bool {
        self.state == ParserState::Done
    }

    fn valid_key_char(char: u8) -> bool {
        char.is_ascii_lowercase()
            || char.is_ascii_uppercase()
            || char.is_ascii_digit()
            || VALID_SYMBOLS.contains(&char)
    }

    fn valid_key(key: &[u8]) -> bool {
        if key.is_empty() {
            return false;
        }

        key.iter().all(|&char| Self::valid_key_char(char))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        headers::{Headers, HeadersParser},
        request::ParseError,
    };

    #[test]
    fn test_valid_single_header() {
        let data = "Host: localhost:42069\r\n\r\n".as_bytes();
        let mut parser = HeadersParser::new();
        assert_eq!(parser.parse(data), Ok(data.len()));
        let mut headers = Headers::new();
        headers.insert("Host", String::from("localhost:42069"));

        assert_eq!(headers, parser.inner_headers());
    }

    #[test]
    fn test_incomplete_header() {
        let data = "Host: localhost:42069\r".as_bytes();
        let mut parser = HeadersParser::new();
        assert_eq!(parser.parse(data), Ok(0));
        assert!(!parser.done());
    }

    #[test]
    fn test_invalid_header_spacing() {
        let data = "Host :localhost:42069\r\n\r\n".as_bytes();
        let mut parser = HeadersParser::new();
        assert_eq!(parser.parse(data), Err(ParseError::InvalidFieldLineSpacing));
        assert!(!parser.done());
    }

    #[test]
    fn test_multiple_headers() {
        let data = "Host: localhost:42069\r\n\
            Content-Type: plain/text\r\n\
            Content-Length: 40\r\n\r\n"
            .as_bytes();

        let mut parser = HeadersParser::new();
        assert_eq!(parser.parse(data), Ok(data.len()));

        let mut headers = Headers::new();
        headers.insert("Host", String::from("localhost:42069"));
        headers.insert("Content-Type", String::from("plain/text"));
        headers.insert("Content-Length", String::from("40"));

        assert_eq!(headers, parser.inner_headers());
    }

    #[test]
    fn test_multiple_value_header() {
        let data = "Key: Value1\r\n\
            Key: Value2\r\n\
            Key: Value3\r\n\r\n"
            .as_bytes();

        let mut parser = HeadersParser::new();
        assert_eq!(parser.parse(data), Ok(data.len()));

        let mut headers = Headers::new();
        headers.insert("Key", String::from("Value1, Value2, Value3"));

        assert_eq!(headers, parser.inner_headers());
    }
}
