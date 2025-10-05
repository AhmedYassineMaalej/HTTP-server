use std::io::Read;

use crate::{
    headers::{Headers, HeadersParser},
    request_line::{RequestLine, RequestLineParser},
};

pub const SEPARATOR: &[u8; 2] = b"\r\n";

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MalformedRequestLine,
    UnsupportedHTTPVersion,
    InvalidFieldLineSpacing,
    MissingFieldLineColumn,
    InvalidTokenCharacter,
    BodyShorterThanReported,
    InvalidHeaderValue,
    IoError,
}

#[derive(Debug, PartialEq, Default)]
pub enum ParserState {
    #[default]
    Init,
    Waiting,
    Done,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, PartialEq)]
pub struct Request {
    pub request_line: RequestLine,
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Request {
    /// Reads and parses data from reader into `Request` struct
    ///
    /// # Errors
    ///
    /// This function will return a `ParseError` if the underlying parser fails
    pub fn from_reader(reader: &mut impl Read) -> Result<Self, ParseError> {
        let mut parser = RequestParser::new(reader);
        parser.parse_request()
    }
}

pub struct RequestParser<R: Read> {
    buf: [u8; 1024],
    buf_len: usize,
    reader: R,
}

impl<R: Read> RequestParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            buf: [0; 1024],
            reader,
            buf_len: 0,
        }
    }

    /// Parses the internal buffer into a request
    ///
    /// # Errors
    /// This function will return `Err(Error)` if it encounters an error while parsing
    pub fn parse_request(&mut self) -> Result<Request, ParseError> {
        let request_line = self.parse_request_line()?;
        let headers = self.parse_headers()?;
        let body_len = match headers.get("Content-Length") {
            Some(len) => len.parse().map_err(|_| ParseError::InvalidHeaderValue)?,
            None => 0,
        };

        let body = self.parse_body(body_len)?;
        Ok(Request {
            request_line,
            headers,
            body,
        })
    }

    fn read_to_buf(&mut self) -> usize {
        let n = self.reader.read(&mut self.buf[self.buf_len..]).unwrap();
        self.buf_len += n;
        n
    }

    fn consume_buf(&mut self, consumed: usize) {
        self.buf.copy_within(consumed..self.buf_len, 0);
        self.buf_len -= consumed;
    }

    fn parse_request_line(&mut self) -> Result<RequestLine, ParseError> {
        let mut parser = RequestLineParser::new();
        loop {
            let consumed = parser.parse(&self.buf[..self.buf_len])?;
            self.consume_buf(consumed);

            if parser.done() {
                break;
            }

            // read more
            self.read_to_buf();
        }

        let request_line = parser.inner();
        Ok(request_line)
    }

    fn parse_headers(&mut self) -> Result<Headers, ParseError> {
        let mut parser = HeadersParser::new();
        loop {
            let consumed = parser.parse(&self.buf[..self.buf_len])?;
            self.consume_buf(consumed);

            if parser.done() {
                break;
            }

            // read more
            self.read_to_buf();
        }

        let request_line = parser.inner_headers();
        Ok(request_line)
    }

    fn parse_body(&mut self, body_len: usize) -> Result<Vec<u8>, ParseError> {
        while self.buf_len < body_len {
            let n = self.read_to_buf();

            if n == 0 {
                return Err(ParseError::BodyShorterThanReported);
            }
        }

        Ok(self.buf[..body_len].to_vec())
    }
}

#[cfg(test)]
pub mod tests {
    use super::{ParseError, Request};
    use crate::{chunk_reader::ChunkReader, headers::Headers, request_line::RequestLine};

    #[test]
    fn test_standard_request() {
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

        let mut headers = Headers::new();
        headers.insert("Host", String::from("localhost:42069"));
        headers.insert("Content-Length", String::from("13"));
        assert_eq!(
            request.unwrap(),
            Request {
                request_line: RequestLine {
                    http_version: String::from("1.1"),
                    request_target: String::from("/submit"),
                    method: String::from("POST"),
                },
                headers,
                body: "hello world!\n".as_bytes().to_vec(),
            },
        );
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
        assert_eq!(request, Err(ParseError::BodyShorterThanReported));
    }
}
