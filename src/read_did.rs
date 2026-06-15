use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

pub struct DidFileReader {
    reader: BufReader<File>,
    position: u64,
}

impl DidFileReader {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::open(path).context("Unable to open Did file")?;
        Ok(DidFileReader {
            reader: BufReader::new(file),
            position: 0,
        })
    }

    pub fn read_did(&mut self) -> Result<Option<String>> {
        let mut line = String::new();
        self.reader
            .seek(SeekFrom::Start(self.position))
            .context("Failed to seek to position")?;
        let bytes_read = self
            .reader
            .read_line(&mut line)
            .context("Failed to read line")?;
        if bytes_read == 0 {
            return Ok(None); // EOF
        }
        self.position = self
            .reader
            .seek(SeekFrom::Current(0))
            .context("Failed to get position")?;
        Ok(Some(line.trim().to_string()))
    }
}
