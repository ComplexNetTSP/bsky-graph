use crate::{AtProtoGetFollows, DidFileReader, Follows};
use anyhow::Result;
use arrow::{array::RecordBatch, datatypes::FieldRef};
use atrium_api::types::string::{AtIdentifier, Did};
use chrono::Local;
use indicatif::ProgressBar;
use parquet::{arrow::ArrowWriter, file::properties::WriterProperties};
use serde_arrow::schema::{SchemaLike, TracingOptions};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
use tokio::time::sleep;
pub struct FollowsWriter {
    atproto: AtProtoGetFollows,
    reader: DidFileReader,
    pub buf_size: usize,
    pub output_dir: String,
    pub buf: Vec<Follows>,
}

impl FollowsWriter {
    pub fn new(
        atproto: AtProtoGetFollows,
        reader: DidFileReader,
        buf_size: usize,
        output_dir: &str,
    ) -> Self {
        FollowsWriter {
            atproto,
            reader,
            buf_size,
            output_dir: output_dir.to_string(),
            buf: vec![],
        }
    }

    pub async fn write_follows(&mut self) -> Result<()> {
        println!("Retrives Flollows:");
        let bar = ProgressBar::new(count_lines(&self.reader.path)? as u64);
        while let Some(did) = self.reader.read_did()? {
            // pause 100 ms in order to pace teh number of request and avoid to be ban
            pause().await;
            // Create from a handle string
            let did = AtIdentifier::Did(Did::new(did).expect("failed to parse Did"));
            let (subject, follows) = self.atproto.get_follows_w_retry(did, 10).await?;
            let mut subject_follows_edge = Follows::create_edge_list(subject, follows)?;
            self.buf.append(&mut subject_follows_edge);
            if self.buf.len() > self.buf_size {
                self.flush()?;
            }
            bar.inc(1);
        }
        self.flush()?;
        bar.finish();
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.buf.len() == 0 {
            return Ok(());
        }
        //eprintln!("flush buffer of size {}", self.buf.len());
        // Determine Arrow schema
        let fields = Vec::<FieldRef>::from_type::<Follows>(TracingOptions::default())?;
        // Build a record batch
        let batch = serde_arrow::to_record_batch(&fields, &self.buf)?;
        self.to_parquet(batch)?;
        self.buf.clear();
        Ok(())
    }

    fn to_parquet(&mut self, batch: RecordBatch) -> Result<()> {
        let timestamp = Local::now().format("%Y_%m_%d_%H_%M_%S").to_string();
        let filepath = format!("{}/follows/follows_{}.parquet", &self.output_dir, timestamp);
        let file = Path::new(&filepath);
        if let Some(parent) = Path::new(file).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(file)?;
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;
        Ok(())
    }
}

async fn pause() {
    sleep(tokio::time::Duration::from_millis(100)).await;
}

fn count_lines(path: &str) -> Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}
