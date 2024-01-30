// Filename helper
// (c) 2024 Ross Younger

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::Context;

/// Holds a filename and provides utility methods for that name
#[derive(Clone, Debug, PartialEq)]
pub struct Filename {
    filename: String,
}

impl Filename {
    /// Standard constructor
    #[must_use]
    pub fn new(filename: &str) -> Self {
        Filename {
            filename: String::from(filename),
        }
    }
    /// Truncatingly opens the given file for writing and returns a buffered write handle.
    /// You should call flush() before dropping the handle.
    pub fn write_handle(&self) -> anyhow::Result<Box<dyn Write>> {
        if self.filename == "-" {
            // stdout is buffered already
            Ok(Box::new(std::io::stdout()))
        } else {
            let path = Path::new(&self.filename);
            let file = File::create(path).with_context(|| "Could not open output file")?;
            let bw = Box::new(BufWriter::new(file));
            Ok(bw)
        }
    }

    /// Truncatingly opens the given file for writing and returns a buffered write handle.
    /// You should call flush() before dropping the handle.
    pub fn open_for_writing(filename: &str) -> anyhow::Result<Box<dyn Write>> {
        Filename::new(filename).write_handle()
    }
}

impl Default for Filename {
    fn default() -> Self {
        Self {
            filename: String::from("-"),
        }
    }
}
