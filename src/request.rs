use std::io::Read;

use crate::{
    headers::{Headers, HeadersParser},
    request_line::{RequestLine, RequestLineParser},
};

pub const SEPARATOR: &[u8; 2] = b"\r\n";

#[derive(Debug, PartialEq)]
pub enum ParserState {
    Init,
    Waiting,
    Done,
}

#[derive(Debug)]
pub struct Request {
    pub request_line: RequestLine,
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Request {
    pub fn from_reader(reader: &mut impl Read) -> Result<Self, Error> {
        let mut buf = [0u8; 1024];
        let mut buf_len = 0;

        let n = reader.read(&mut buf[buf_len..]).unwrap();
        buf_len += n;

        let mut parser = RequestLineParser::new();
        loop {
            let consumed = parser.parse(&buf[..buf_len])?;
            buf.copy_within(consumed..buf_len, 0);
            buf_len -= consumed;

            if parser.done() {
                break;
            }

            // read more
            let n = reader.read(&mut buf[buf_len..]).unwrap();
            buf_len += n;
        }

        let request_line = parser.inner();

        let mut parser = HeadersParser::new();
        loop {
            let consumed = parser.parse(&buf[..buf_len])?;
            buf.copy_within(consumed..buf_len, 0);
            buf_len -= consumed;

            if parser.done() {
                break;
            }

            // read more
            let n = reader.read(&mut buf[buf_len..]).unwrap();
            buf_len += n;
        }

        let headers = parser.inner_headers();

        // parse body
        let body_length: usize = headers
            .get("Content-Length")
            .map(|s| s.parse().unwrap())
            .unwrap_or(0);

        while buf_len < body_length {
            let n = reader.read(&mut buf[buf_len..]).unwrap();

            if n == 0 {
                return Err(Error::BodyShorterThanReported);
            }

            buf_len += n;
        }

        let body = buf[..body_length].to_vec();

        Ok(Request {
            request_line,
            headers,
            body,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    MalformedRequestLine,
    UnsupportedHTTPVersion,
    InvalidFieldLineSpacing,
    MissingFieldLineColumn,
    InvalidTokenCharacter,
    BodyShorterThanReported,
}

#[cfg(test)]
pub mod tests {
    use super::{Error, Request};
    use crate::chunk_reader::ChunkReader;

    #[test]
    fn test_standard() {
        let mut data = ChunkReader::new(
            "POST /submit HTTP/1.1\r\n\
            Host: localhost:42069\r\n\
            Content-Length: 13\r\n\
            \r\n\
            hello world!\n",
            3,
        );

        let request = Request::from_reader(&mut data);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.body, "hello world!\n".as_bytes());
    }

    #[test]
    fn test_short_body() {
        let mut data = ChunkReader::new(
            "POST /submit HTTP/1.1\r\n\
            Host: localhost:42069\r\n\
            Content-Length: 20\r\n\
            \r\n\
            hello world!\n",
            3,
        );

        let request = Request::from_reader(&mut data);
        assert!(request.is_err());
        let error = request.unwrap_err();
        assert_eq!(error, Error::BodyShorterThanReported)
    }
}
