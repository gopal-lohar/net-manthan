use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// Frame format: [length: u32][data: bytes]
#[derive(Debug)]
pub struct MessageCodec;

impl Decoder for MessageCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let length = u32::from_le_bytes([src[0], src[1], src[2], src[3]]) as usize;

        if length > 1024 * 1024 {
            // 1MB limit
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Message too large",
            ));
        }

        if src.len() < 4 + length {
            return Ok(None);
        }

        src.advance(4);
        let data = src.split_to(length);
        Ok(Some(data))
    }
}

impl Encoder<Vec<u8>> for MessageCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let length = item.len() as u32;
        dst.reserve(4 + item.len());
        dst.put_u32_le(length);
        dst.extend_from_slice(&item);
        Ok(())
    }
}
