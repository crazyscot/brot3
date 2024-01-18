// Filename helper
// (c) 2024 Ross Younger

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(PartialEq)]
pub struct Filename {
    filename: String,
}

impl Filename {
    pub fn new(filename: &str) -> Self {
        Filename {
            filename: String::from(filename),
        }
    }
    pub fn get(&self) -> &String {
        &self.filename
    }
    /// Truncatingly opens the given file for writing and returns a buffered write handle.
    /// You should call flush() before dropping the handle.
    pub fn write_handle(&self) -> std::io::Result<Box<dyn Write>> {
        if self.filename == "-" {
            // stdout is buffered already
            Ok(Box::new(std::io::stdout()))
        } else {
            let path = Path::new(&self.filename);
            let file = File::create(path)?;
            let bw = Box::new(BufWriter::new(file)) as Box<dyn Write>;
            Ok(bw)
        }
    }
}
