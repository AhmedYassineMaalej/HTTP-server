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

    pub fn parse_request(&mut self) -> Result<Request, Error> {
        let request_line = self.parse_request_line()?;
        let headers = self.parse_headers()?;
        let body_len = headers
            .get("Content-Length")
            .map(|s| s.parse().unwrap())
            .unwrap_or(0);
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

    fn parse_request_line(&mut self) -> Result<RequestLine, Error> {
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

    pub fn parse_headers(&mut self) -> Result<Headers, Error> {
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

    fn parse_body(&mut self, body_len: usize) -> Result<Vec<u8>, Error> {
        while self.buf_len < body_len {
            let n = self.read_to_buf();

            if n == 0 {
                return Err(Error::BodyShorterThanReported);
            }
        }

        Ok(self.buf[..body_len].to_vec())
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
