use std::{io::SeekFrom, path::PathBuf};
use tokio::{
    fs::{File, OpenOptions},
    io::{self, AsyncSeekExt},
};

use crate::buf_writer_on_flush::BufWriterWithOnFlush;

pub async fn open_file_writer(
    file: PathBuf,
    seek: u64,
    buf_size: usize,
    on_flush: Box<dyn FnMut(usize) + Send>,
) -> Result<BufWriterWithOnFlush<File>, io::Error> {
    // Create custom file that notifies on flush
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file)
        .await?;

    let mut writer = BufWriterWithOnFlush::with_capacity(buf_size, file, on_flush);

    writer.seek(SeekFrom::Start(seek)).await?;

    Ok(writer)
}
