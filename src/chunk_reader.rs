#![cfg(test)]

use std::io::Read;

pub struct ChunkReader {
    data: String,
    chunk_size: usize,
    pos: usize,
}

impl ChunkReader {
    pub fn new(data: &str, chunk_size: usize) -> Self {
        Self {
            data: String::from(data),
            chunk_size: chunk_size,
            pos: 0,
        }
    }
}

impl Read for ChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.data.len() {
            return Ok(0);
        }

        let read_size = self.chunk_size.min(buf.len());
        let end_index = (self.pos + read_size).min(self.data.len());
        let n = end_index - self.pos;
        buf[..n].copy_from_slice(&self.data.as_bytes()[self.pos..end_index]);

        self.pos += n;
        Ok(n)
    }
}
