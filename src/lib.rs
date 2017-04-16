extern crate bytes;
extern crate serde;
extern crate serde_cbor;
extern crate tokio_io;

use std::io::{ErrorKind, Read, Result as IoResult};
use std::marker::PhantomData;

use bytes::BytesMut;
use serde::Deserialize;
use serde_cbor::de::Deserializer;
use serde_cbor::error::Error as CborError;
use tokio_io::codec::Decoder as IoDecoder;

/// A `Read` wrapper that also counts the used bytes.
///
/// This wraps a `Read` into another `Read` that keeps track of how many bytes were read. This is
/// needed, as there's no way to get the position out of the CBOR decoder.
struct Counted<'a, R: 'a> {
    r: &'a mut R,
    pos: &'a mut usize,
}

impl<'a, R: Read> Read for Counted<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match self.r.read(buf) {
            Ok(size) => {
                *self.pos += size;
                Ok(size)
            },
            e => e,
        }
    }
}

/// CBOR based decoder.
///
/// This decoder can be used with `tokio_io`'s `Framed` to decode CBOR encoded frames. Anything
/// that is `serde`s `Deserialize` can be decoded this way.
pub struct Decoder<Item> {
    _data: PhantomData<*const Item>,
}

impl<Item: Deserialize> Decoder<Item> {
    /// Creates a new decoder.
    pub fn new() -> Self {
        Decoder { _data: PhantomData }
    }
}

impl<Item: Deserialize> IoDecoder for Decoder<Item> {
    type Item = Item;
    type Error = CborError;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Item>, CborError> {
        // Try to read the value using the Cbor's deserializer, but keep track of how many data has
        // been eaten.
        let mut pos = 0;
        let result = {
            let mut slice: &[u8] = src;
            let reader = Counted {
                r: &mut slice,
                pos: &mut pos,
            };
            // Use the deserializer directly, instead of using `deserialize_from`. We explicitly do
            // *not* want to check that there are no trailing bytes ‒ there may be, and they are
            // the next frame.
            let mut deserializer = Deserializer::new(reader);
            Item::deserialize(&mut deserializer)
        };
        match result {
            // If we read the item, we also need to consume the corresponding bytes.
            Ok(item) => {
                src.split_to(pos);
                Ok(Some(item))
            },
            // If it errors on not enough bytes, then we just signal we want to be called next
            // time.
            Err(CborError::Eof) => Ok(None),
            // Sometimes the EOF is signalled as IO error
            Err(CborError::Io(ref io)) if io.kind() == ErrorKind::UnexpectedEof => Ok(None),
            // Any other error is simply passed through.
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_cbor;

    use super::*;

    /// Try decoding CBOR based data.
    #[test]
    fn decode() {
        let mut data = HashMap::new();
        data.insert("hello".to_owned(), 42usize);
        data.insert("world".to_owned(), 0usize);
        let encoded = serde_cbor::to_vec(&data).unwrap();
        let mut all = BytesMut::with_capacity(128);
        // Put two copies and a bit into the buffer
        all.extend(&encoded);
        all.extend(&encoded);
        all.extend(&encoded[..1]);
        // Create the decoder
        let mut decoder: Decoder<HashMap<String, usize>> = Decoder::new();
        // We can now decode the first two copies
        let decoded = decoder.decode(&mut all).unwrap().unwrap();
        assert_eq!(data, decoded);
        let decoded = decoder.decode(&mut all).unwrap().unwrap();
        assert_eq!(data, decoded);
        // And only 1 byte is left
        assert_eq!(1, all.len());
        // But the third one is not ready yet, so we get Ok(None)
        assert!(decoder.decode(&mut all).unwrap().is_none());
        // That single byte should still be there, yet unused
        assert_eq!(1, all.len());
        // We add the rest and get a third copy
        all.extend(&encoded[1..]);
        let decoded = decoder.decode(&mut all).unwrap().unwrap();
        assert_eq!(data, decoded);
        // Nothing there now
        assert!(all.is_empty());
        // Now we put some garbage there and see that it errors
        all.extend(&[0, 1, 2, 3, 4]);
        decoder.decode(&mut all).unwrap_err();
        // All 5 bytes are still there
        assert_eq!(5, all.len());
    }
}
