// Filename helper
// (c) 2024 Ross Younger

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
    pub fn write_handle(&self) -> std::io::Result<Box<dyn std::io::Write>> {
        if self.filename == "-" {
            Ok(Box::new(std::io::stdout()))
        } else {
            Ok(Box::new(std::fs::File::create(&self.filename)?))
        }
    }
}
