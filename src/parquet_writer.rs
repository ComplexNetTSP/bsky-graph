use crate::{DidFileReader, Follows, atproto_client::AtProtoClient, utils::count_lines};
use anyhow::Result;
use arrow::{array::RecordBatch, datatypes::FieldRef};
use atrium_api::types::string::{AtIdentifier, Did};
use chrono::Local;
use indicatif::ProgressBar;
use log::{error, info};
use parquet::{arrow::ArrowWriter, basic::Compression, file::properties::WriterProperties};
use serde::{Deserialize, Serialize};
use serde_arrow::schema::{SchemaLike, TracingOptions};
use std::{fs::File, path::Path};
use tokio::time::sleep;

// to use only in async function
macro_rules! pause_ms {
    ($t: expr) => {
        sleep(tokio::time::Duration::from_millis($t)).await;
    };
}

pub struct ParquetWriter<T, C>
where
    T: Serialize + Deserialize<'static>,
    C: AtProtoClient<T>,
{
    atproto: C,
    reader: DidFileReader,
    pub buf_size: usize,
    pub output_dir: String,
    pub buf: Vec<T>,
}

impl<T, C> ParquetWriter<T, C>
where
    T: Serialize + Deserialize<'static>,
    C: AtProtoClient<T>,
{
    pub fn new(atproto: C, reader: DidFileReader, buf_size: usize, output_dir: &str) -> Self {
        ParquetWriter {
            atproto,
            reader,
            buf_size,
            output_dir: output_dir.to_string(),
            buf: vec![],
        }
    }

    pub async fn write(&mut self) -> Result<()> {
        println!("Retrives Flollows:");
        let bar = ProgressBar::new(count_lines(&self.reader.path)? as u64);
        while let Some(did) = self.reader.read_did()? {
            // pause 100 ms in order to pace the number of request and avoid to be ban
            pause_ms!(100);
            // Create from a handle string
            let did_parsed = match Did::new(did) {
                Ok(d) => d,
                Err(e) => {
                    error!("Unable to parse did: {}", e);
                    continue;
                }
            };
            let did_id: AtIdentifier = AtIdentifier::Did(did_parsed);
            let mut edges = self.atproto.get_graph_w_retry(did_id, 10).await?;
            self.buf.append(&mut edges);
            if self.buf.len() > self.buf_size {
                self.flush()?;
            }
            bar.inc(1);
        }
        self.flush()?;
        bar.finish();
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        info!(
            "Process follow_writer flush buffer of size {}",
            self.buf.len()
        );
        // Determine Arrow schema
        let fields = Vec::<FieldRef>::from_type::<Follows>(TracingOptions::default())?;
        // Build a record batch
        let batch = serde_arrow::to_record_batch(&fields, &self.buf)?;
        self.to_parquet(batch)?;
        self.buf.clear();
        Ok(())
    }

    fn to_parquet(&self, batch: RecordBatch) -> Result<()> {
        let timestamp = Local::now().format("%Y_%m_%d_%H_%M_%S").to_string();
        let filepath = format!("{}/follows/follows_{}.parquet", &self.output_dir, timestamp);
        info!("Process follow_writer write file: {}", &filepath);
        let file = Path::new(&filepath);
        if let Some(parent) = Path::new(file).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(file)?;
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();
        let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;
        Ok(())
    }
}
