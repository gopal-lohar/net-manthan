// the default buffer writers doesn't have a reliable way to do something whenever there is a write in the file system, (updating the download progress)
// extending the async BufWriter or Writer to achieve this functionality is weird and complicated
// this is direct adaptation of std::io::BufWriter's logic with tokio and for our usecase, not generic

// the write_all function returns std::io::Result<Option<bytes_written>>

use tokio::io::AsyncWriteExt;

pub struct CustomWriter {
    buf: Vec<u8>,
    inner: tokio::fs::File,
}

// core buffers implementations
impl CustomWriter {
    pub async fn with_capacity(inner: tokio::fs::File, capacity: usize) -> Self {
        CustomWriter {
            buf: Vec::with_capacity(capacity),
            inner,
        }
    }

    fn spare_capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
    }

    // SAFETY: Requires `buf.len() <= self.buf.capacity() - self.buf.len()`,
    // i.e., that input buffer length is less than or equal to spare capacity.
    unsafe fn write_to_buffer_unchecked(&mut self, buf: &[u8]) {
        debug_assert!(buf.len() <= self.spare_capacity());
        let old_len = self.buf.len();
        let buf_len = buf.len();
        let src = buf.as_ptr();
        unsafe {
            let dst = self.buf.as_mut_ptr().add(old_len);
            std::ptr::copy_nonoverlapping(src, dst, buf_len);
            self.buf.set_len(old_len + buf_len);
        }
    }

    pub async fn flush_buf(&mut self) -> std::io::Result<Option<u64>> {
        self.inner.write_all(&self.buf).await?;
        let bytes_written = self.buf.len();
        self.buf.clear();
        Ok(Some(bytes_written as u64))
    }

    pub async fn flush(&mut self) -> std::io::Result<Option<u64>> {
        match self.flush_buf().await {
            Ok(bytes_flushed) => self.inner.flush().await.map(|_| bytes_flushed),
            Err(err) => Err(err),
        }
    }

    pub async fn write_all_cold(&mut self, buf: &[u8]) -> std::io::Result<Option<u64>> {
        let mut bytes_written = Some(0u64);
        if buf.len() > self.spare_capacity() {
            bytes_written = self.flush_buf().await?;
        }

        if buf.len() >= self.buf.capacity() {
            self.inner.write_all(buf).await?;
            Ok(Some(match bytes_written {
                Some(bytes_written) => bytes_written + buf.len() as u64,
                None => buf.len() as u64,
            }))
        } else {
            // Write to the buffer. In this case, we write to the buffer even if it fills it
            // exactly. Doing otherwise would mean flushing the buffer, then writing this
            // input to the inner writer, which in many cases would be a worse strategy.

            // SAFETY: There was either enough spare capacity already, or there wasn't and we
            // flushed the buffer to ensure that there is. In the latter case, we know that there
            // is because flushing ensured that our entire buffer is spare capacity, and we entered
            // this block because the input buffer length is less than that capacity. In either
            // case, it's safe to write the input buffer to our buffer.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(bytes_written)
        }
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<Option<u64>> {
        if buf.len() < self.spare_capacity() {
            // SAFETY: safe by above conditional.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(None)
        } else {
            self.write_all_cold(buf).await
        }
    }
}
