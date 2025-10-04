use std::{
    collections::{HashMap, hash_map::Iter},
    io::Read,
};

use crate::request::{Error, ParserState, SEPARATOR};

const VALID_SYMBOLS: [u8; 15] = [
    b'!', b'#', b'$', b'%', b'&', b'\'', b'*', b'+', b'-', b'.', b'^', b'_', b'`', b'|', b'~',
];

#[derive(Debug)]
pub struct Headers {
    inner: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
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

    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(&key.to_lowercase())
    }

    pub fn from_reader(mut reader: impl Read) -> Result<Headers, Error> {
        let mut parser = HeadersParser::new();
        let mut buf = [0u8; 1024];
        let mut buf_len = 0;

        while !parser.done() {
            let n = reader.read(&mut buf[buf_len..]).unwrap();
            buf_len += n;

            let consumed = parser.parse(&buf[..buf_len])?;
            buf.copy_within(consumed..buf_len, 0);
            buf_len -= consumed;
        }

        Ok(parser.inner_headers())
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.inner.iter()
    }

    pub fn get_default(content_length: usize) -> Headers {
        let mut headers = Self::new();

        // headers.insert("Content-Length", content_length.to_string());
        headers.insert("Transfer-Encoding", String::from("chunked"));
        // headers.insert("Connection", String::from("close"));
        headers.insert("Content-Type", String::from("text/plain"));

        headers
    }
}

pub struct HeadersParser {
    headers: Headers,
    state: ParserState,
}

impl HeadersParser {
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
            state: ParserState::Init,
        }
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.state = ParserState::Init;
        let mut read = 0;

        while self.state == ParserState::Init {
            read += self.parse_header(&data[read..])?;
        }

        Ok(read)
    }

    fn parse_header(&mut self, data: &[u8]) -> Result<usize, Error> {
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
            return Err(Error::MissingFieldLineColumn);
        };

        let (key, value) = (&line[..colon_idx], &line[colon_idx + 1..]);

        if key.ends_with(b" ") {
            return Err(Error::InvalidFieldLineSpacing);
        }

        if !Self::valid_key(key) {
            return Err(Error::InvalidTokenCharacter);
        }

        let key = String::from_utf8_lossy(key.trim_ascii()).into_owned();
        let value = String::from_utf8_lossy(value.trim_ascii()).into_owned();

        self.headers.insert(&key, value);

        Ok(read)
    }

    pub fn inner_headers(self) -> Headers {
        self.headers
    }

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
    use super::Headers;
    use crate::chunk_reader::ChunkReader;
    use crate::headers::HeadersParser;
    use crate::request::{Error, RequestParser};

    #[test]
    fn test_valid_single_header() {
        let data = "Host: localhost:42069\r\n\r\n".as_bytes();
        let mut parser = HeadersParser::new();
        assert_eq!(parser.parse(data), Ok(25));
        let headers = parser.inner_headers();
        assert_eq!(headers.get("Host"), Some(&String::from("localhost:42069")));
    }

    #[test]
    fn test_incomplete_header() {
        let data = "Host: localhost:42069\r".as_bytes();
        let mut parser = HeadersParser::new();
        assert_eq!(parser.parse(data), Ok(0));
        assert!(!parser.done());
    }

    #[test]
    fn test_invalid_spacing() {
        // invalid spacing
        let data = ChunkReader::new("    Host : localhost:42069      \r\n\r\n", 8);
        let headers = Headers::from_reader(data);
        assert!(headers.is_err());
        assert_eq!(headers.unwrap_err(), Error::InvalidFieldLineSpacing);
    }

    #[test]
    fn test_missing_column() {
        // invalid spacing
        let data = ChunkReader::new("Host   localhost\r\n\r\n", 8);

        let mut parser = RequestParser::new(data);
        let headers = parser.parse_headers();

        assert!(headers.is_err());
        assert_eq!(headers.unwrap_err(), Error::MissingFieldLineColumn);
    }

    #[test]
    fn test_multiple() {
        let data = ChunkReader::new("Host: localhost:42069\r\nName: Maalej\r\n\r\n", 5);

        let mut parser = RequestParser::new(data);
        let headers = parser.parse_headers();

        assert!(headers.is_ok());
        let headers = headers.unwrap();
        assert_eq!(headers.get("Host"), Some(&String::from("localhost:42069")));
        assert_eq!(headers.get("Name"), Some(&String::from("Maalej")));
        assert!(headers.get("NonExistantKey").is_none());
    }

    #[test]
    fn test_invalid_characters() {
        let data = ChunkReader::new("H©st: localhost:42069\r\n\r\n", 5);

        let mut parser = RequestParser::new(data);
        let headers = parser.parse_headers();

        assert!(headers.is_err());
    }

    #[test]
    fn test_multiple_values() {
        let data = ChunkReader::new(
            "Set-Person: lane-loves-go\r\nSet-Person: prime-loves-zig\r\nSet-Person: tj-loves-ocaml\r\n\r\n",
            6,
        );

        let mut parser = RequestParser::new(data);
        let headers = parser.parse_headers();

        assert!(headers.is_ok());
        let headers = headers.unwrap();

        assert_eq!(
            headers.get("Set-Person"),
            Some(&String::from(
                "lane-loves-go, prime-loves-zig, tj-loves-ocaml"
            ))
        );
    }
}
