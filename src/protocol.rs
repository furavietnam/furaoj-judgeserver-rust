// Logic: Wire protocol encoder and decoder matching FuraOJ Bridge 4-byte BE framing and zlib compression.
// Input: Serialized JSON payloads or compressed wire byte slices.
// Output: Wire-encoded byte buffers or decoded JSON payload strings.

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

// Logic: Compresses JSON payload using zlib and prepends a 4-byte big-endian unsigned length header.
// Input: payload (&str JSON string).
// Output: Result<Vec<u8>, std::io::Error> containing wire packet.
pub fn encode_packet(payload: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(payload.as_bytes())?;
    let compressed = encoder.finish()?;

    let len = compressed.len() as u32;
    let mut packet = Vec::with_capacity(4 + compressed.len());
    packet.extend_from_slice(&len.to_be_bytes());
    packet.extend_from_slice(&compressed);

    Ok(packet)
}

// Logic: Decompresses zlib-compressed wire packet payload into original UTF-8 JSON text.
// Input: compressed_data (&[u8] compressed slice).
// Output: Result<String, std::io::Error> containing decoded UTF-8 string.
pub fn decode_packet(compressed_data: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut decoded = String::new();
    decoder.read_to_string(&mut decoded)?;
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_roundtrip() {
        let msg = r#"{"name":"ping-response","timestamp":1700000000}"#;
        let encoded = encode_packet(msg).expect("encode packet");
        assert!(encoded.len() > 4);

        let len = u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]) as usize;
        assert_eq!(len, encoded.len() - 4);

        let decoded = decode_packet(&encoded[4..]).expect("decode packet");
        assert_eq!(msg, decoded);
    }
}
