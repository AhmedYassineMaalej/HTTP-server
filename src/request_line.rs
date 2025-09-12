use std::io::Read;

use crate::request::{Error, ParserState, SEPARATOR};

#[derive(Debug)]
pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

impl RequestLine {
    pub fn from_reader(reader: &mut impl Read) -> Result<RequestLine, Error> {
        let mut parser = RequestLineParser::new();
        let mut buf = [0u8; 1024];
        let mut buf_len = 0;

        while !parser.done() {
            let n = reader.read(&mut buf[buf_len..]).unwrap();
            buf_len += n;

            let consumed = parser.parse(&buf[..buf_len])?;
            buf.copy_within(consumed..buf_len, 0);
            buf_len -= consumed;
        }

        Ok(parser.inner())
    }
}

pub struct RequestLineParser {
    state: ParserState,
    request_line: Option<RequestLine>,
}

impl RequestLineParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Init,
            request_line: None,
        }
    }

    fn parse_request_line(&mut self, b: &[u8]) -> Result<usize, Error> {
        let Some(idx) = b
            .windows(SEPARATOR.len())
            .position(|window| window == SEPARATOR)
        else {
            self.state = ParserState::Waiting;
            return Ok(0);
        };

        let start_line = &b[..idx];
        let read = idx + SEPARATOR.len();

        let mut parts = start_line.split(|byte| byte == &b' ');

        let (Some(method), Some(request_target), Some(http_version)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return Err(Error::MalformedRequestLine);
        };

        if http_version != b"HTTP/1.1" {
            return Err(Error::UnsupportedHTTPVersion);
        }

        let request_line = RequestLine {
            http_version: String::from("1.1"),
            request_target: String::from_utf8_lossy(request_target).into_owned(),
            method: String::from_utf8_lossy(method).into_owned(),
        };

        self.request_line = Some(request_line);
        self.state = ParserState::Done;
        Ok(read)
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.state = ParserState::Init;
        let mut read = 0;

        loop {
            match self.state {
                ParserState::Init => {
                    read += self.parse_request_line(&data[read..])?;
                }

                ParserState::Done | ParserState::Waiting => break,
            }
        }

        Ok(read)
    }

    pub fn done(&self) -> bool {
        self.state == ParserState::Done
    }

    pub fn inner(self) -> RequestLine {
        self.request_line.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_reader::ChunkReader;

    #[test]
    fn test_line_parse() {
        // Test: Good GET Request line
        let request_line = RequestLine::from_reader(&mut ChunkReader::new(
            "GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            1,
        ));

        assert!(request_line.is_ok());

        let request_line = request_line.unwrap();
        assert_eq!("GET", request_line.method);
        assert_eq!("/", request_line.request_target);
        assert_eq!("1.1", request_line.http_version);

        // Test: Good GET Request line with path
        let request_line = RequestLine::from_reader(&mut ChunkReader::new(
            "GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            1,
        ));

        assert!(request_line.is_ok());
        let request_line = request_line.unwrap();

        assert_eq!("GET", request_line.method);
        assert_eq!("/coffee", request_line.request_target);
        assert_eq!("1.1", request_line.http_version);

        // Test: Invalid number of parts in request line
        let err = RequestLine::from_reader(&mut ChunkReader::new(
            "/coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n",
            2,
        ));

        assert!(err.is_err())
    }
}
