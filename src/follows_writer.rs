use crate::{AtProtoGetFollows, DidFileReader, Follows};
use anyhow::Result;
use atrium_api::types::string::AtIdentifier;
use atrium_api::types::string::Did;
pub struct FollowsWriter {
    atproto: AtProtoGetFollows,
    reader: DidFileReader,
    buf_size: usize,
    pub buf: Vec<Follows>,
}

impl FollowsWriter {
    pub fn new(atproto: AtProtoGetFollows, reader: DidFileReader, buf_size: usize) -> Self {
        FollowsWriter {
            atproto,
            reader,
            buf_size,
            buf: vec![],
        }
    }

    pub async fn write_follows(&mut self) -> Result<()> {
        const BUFF_SIZE: usize = 100;
        while let Some(did) = self.reader.read_did()? {
            // Create from a handle string
            let did = AtIdentifier::Did(Did::new(did).expect("failed to parse Did"));
            let (subject, follows) = self.atproto.get_follows(did).await?;
            let mut subject_follows_edge = Follows::create_edge_list(subject, follows)?;
            self.buf.append(&mut subject_follows_edge);
            if self.buf.len() > BUFF_SIZE {
                self.flush()?;
            }
        }
        self.flush()?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.buf.len() == 0 {
            return Ok(());
        }
        eprintln!("flush buffer of size {}", self.buf.len());
        self.buf.clear();
        Ok(())
    }
}
