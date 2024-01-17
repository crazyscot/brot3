// Filename helper
// (c) 2024 Ross Younger

use std::fs::File;
use std::io::Write;
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
    pub fn write_handle(&self) -> std::io::Result<Box<dyn Write>> {
        if self.filename == "-" {
            Ok(Box::new(std::io::stdout()))
        } else {
            let path = Path::new(&self.filename);
            let file = File::create(path)?;
            Ok(Box::new(file))
        }
    }
}
