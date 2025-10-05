use crate::request::{ParseError, ParserState, SEPARATOR};

#[derive(Debug)]
pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

#[derive(Default)]
pub struct RequestLineParser {
    state: ParserState,
    request_line: Option<RequestLine>,
}

impl RequestLineParser {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// parse data inside `b` buffer and return amount of bytes consumed
    /// will return `Ok(0)` if no data is consumed
    ///
    /// # Errors
    ///
    /// This function will return:
    /// - `Error::MalformedRequestLine` if the request line is malformed
    /// - `Error::Unsupported HTTPVersion` if the HTTP version is not HTTP/1.1
    fn parse_request_line(&mut self, b: &[u8]) -> Result<usize, ParseError> {
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
            return Err(ParseError::MalformedRequestLine);
        };

        if http_version != b"HTTP/1.1" {
            return Err(ParseError::UnsupportedHTTPVersion);
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

    /// Parse data inside `data` buffer and return amount of bytes consumed
    /// will return `Ok(0)` if not data is consumed
    ///
    /// # Errors
    ///
    /// This function will return:
    /// - `Error::MalformedRequestLine` if the request line is malformed
    /// - `Error::Unsupported HTTPVersion` if the HTTP version is not HTTP/1.1
    pub fn parse(&mut self, data: &[u8]) -> Result<usize, ParseError> {
        self.state = ParserState::Init;
        let mut read = 0;

        while self.state == ParserState::Init {
            read += self.parse_request_line(&data[read..])?;
        }

        Ok(read)
    }

    #[must_use]
    pub fn done(&self) -> bool {
        self.state == ParserState::Done
    }

    /// Transform into inner `RequestLine` struct
    ///
    /// # Panics
    ///
    /// Panics if inner called before `parse_request_line` function
    /// Will panic if `done` returns `false`
    #[must_use]
    pub fn inner(self) -> RequestLine {
        self.request_line.unwrap()
    }
}

#[cfg(test)]
mod tests {}
