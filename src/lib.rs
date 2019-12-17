use std::ffi::OsString;
use std::fs;
use std::io::BufRead;
use std::path::Path;

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

#[derive(PartialEq)]
pub enum Game {
    TESV,
    FO4,
}

pub struct ParserSession {
    src: Source,
    game: Game,
}

impl ParserSession {
    pub fn from_file(path: &str, game: Game) -> Result<Self, String> {
        let path = Path::new(path);
        if !path.is_file() {
            return Err("Path is not a file.".to_string());
        }
        let filename = path.file_name().expect("Could not find file name.").to_owned();
        let content = fs::read_to_string(path).map_err(|x| format!("{}", x))?;
        let src = Source { filename, content };
        Ok(Self { src, game })
    }

    pub fn from_string(script: &str, game: Game) -> Self {
        let filename = OsString::from("<stdin>");
        let src = Source { filename, content: script.to_string() };
        Self { src, game }
    }

    pub fn new_error(&self) -> errors::ErrorBuilder {
        errors::ErrorBuilder::new(self, true)
    }
}
