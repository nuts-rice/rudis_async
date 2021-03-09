//Define RudisFrame which implements Encoder and Decoder traits from tokio-codec
use bytes::Bytes::BytesMuts;
use resp::{Decoder as RespDecoder, Value};
use std::io;
use std::io::BufReader;
use std::str;
use tokio_codec::{Decoder, Encoder};

pub struct RespCodec;
//how-to encode a resp::Value to a stream of bytes to client
impl Encoder for RespCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn encode(&mut self, msg: Vec<u8>, buf: &mut BytesMut) -> io::Result<()> {
        buf.reserve(msg.len());
        buf.extend(msg);
        Ok(())
    }
}

//how-to for parsing incoming bytes into a resp::Value type
impl Decoder for RespCodec {
    type Item = Value;
    type Error = io::Error;

    //Result: Ok + Error, Option: Some + None
    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Value>> {
        let s = if let Some(n) = buf.iter().rposition(|b| *b == b'\n') {
            let client_query = buf.split_to(n + 1);

            match str::from_utf8(&client_query.as_ref()) {
                Ok(s) => s.to_string(),
                Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            }
        } else {
            return Ok(None);
        };
        if let Ok(v) = RespDecoder::new(&mut BufReader::new(s.as_bytes())).decode() {
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
}
