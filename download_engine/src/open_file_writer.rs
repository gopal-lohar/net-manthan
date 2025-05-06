use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncSeekExt, AsyncWrite, AsyncWriteExt, BufWriter, SeekFrom};

// Custom writer that notifies on flush
pub struct FlushNotifyFile {
    file: File,
    on_flush: Box<dyn FnMut() + Send>,
}

impl FlushNotifyFile {
    async fn new(
        path: PathBuf,
        options: &OpenOptions,
        on_flush: impl FnMut() + Send + 'static,
    ) -> io::Result<Self> {
        let file = options.open(path).await?;
        Ok(Self {
            file,
            on_flush: Box::new(on_flush),
        })
    }

    // Add seek method that delegates to the inner file
    async fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos).await
    }
}

// Implement AsyncWrite for our custom file
impl AsyncWrite for FlushNotifyFile {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.file).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut self.file).poll_flush(cx);
        if let Poll::Ready(Ok(_)) = &result {
            (self.on_flush)();
        }
        result
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut self.file).poll_shutdown(cx);
        if let Poll::Ready(Ok(_)) = &result {
            (self.on_flush)();
        }
        result
    }
}

// Now let's create a wrapper for BufWriter that gives us access to seeking
pub struct SeekableBufWriter<W: AsyncWrite> {
    inner: BufWriter<W>,
}

impl<W: AsyncWrite> SeekableBufWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            inner: BufWriter::new(writer),
        }
    }
}

// Forward AsyncWrite implementation to the inner BufWriter
impl<W: AsyncWrite + Unpin> AsyncWrite for SeekableBufWriter<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// Additional implementation for our seekable buffer writer
impl SeekableBufWriter<FlushNotifyFile> {
    // Add method to seek the underlying file
    // This requires flushing the buffer first to ensure consistent state
    async fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        // Flush the buffer first to make sure all data is written
        self.inner.flush().await?;

        // Now that the buffer is flushed, we can safely seek in the underlying file
        let inner_writer = self.inner.get_mut();
        inner_writer.seek(pos).await
    }
}

pub async fn open_file_writer(
    file: PathBuf,
    seek: u64,
    on_flush: impl FnMut() + Send + 'static,
) -> Result<SeekableBufWriter<FlushNotifyFile>, io::Error> {
    // Create custom file that notifies on flush
    let notify_file =
        FlushNotifyFile::new(file, &OpenOptions::new().write(true).create(true), on_flush).await?;

    let mut writer = SeekableBufWriter::new(notify_file);

    writer.seek(SeekFrom::Start(seek)).await?;

    Ok(writer)
}
