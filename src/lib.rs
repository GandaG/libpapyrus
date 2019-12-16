use std::ffi::OsString;
use std::fs;
use std::io::BufRead;

mod errors;

struct Source {
    pub filename: OsString,
    pub content: String,
}

impl Source {
    // (line number, column number)
    fn lineno_from_offset(&self, mut offset: usize) -> (usize, usize) {
        let mut content = &self.content.bytes().collect::<Vec<u8>>()[..];
        let mut buf = String::new();
        let mut length;
        let mut line_num = 1;
        loop {
            content.read_line(&mut buf).unwrap();
            length = buf.len();
            if offset < length {
                break;
            }
            offset -= length;
            line_num += 1;
            buf.clear();
        }
        (line_num, offset)
    }

    fn lines_from_linenos(&self, lo: usize, hi: usize) -> Vec<String> {
        let mut content = &self.content.bytes().collect::<Vec<u8>>()[..];
        let mut buf = String::new();
        let mut line_num = 1;
        let mut lines = vec![];
        loop {
            content.read_line(&mut buf).unwrap();
            if lo <= line_num {
                lines.push(buf.clone());
            }
            if line_num >= hi {
                break;
            }
            line_num += 1;
            buf.clear();
        }
        lines
    }
}

pub struct ParserSession {
    src: Source,
}

impl ParserSession {
    pub fn new_error(&self) -> errors::ErrorBuilder {
        errors::ErrorBuilder::new(self, true)
    }
}
