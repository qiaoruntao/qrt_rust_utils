use std::fs::File;
use std::io::{BufWriter, Write};

pub struct FileUtils {}

impl FileUtils {
    pub fn write_file(file_path: &str, content: &str) {
        let file = File::open(file_path).unwrap();
        let mut writer = BufWriter::new(file);
        let _result = writer.write(content.as_bytes());
    }
}