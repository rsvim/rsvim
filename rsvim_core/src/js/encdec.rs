//! Bytes encoder/decoder for async task results.

const BINCODE_CONFIG: bincode::config::Configuration =
  bincode::config::standard();

pub fn encode_bytes<T>(input: T) -> Vec<u8>
where
  T: bincode::enc::Encode,
{
  bincode::encode_to_vec(input, BINCODE_CONFIG).unwrap()
}

pub fn decode_bytes<T>(bytes: &Vec<u8>) -> (T, usize)
where
  T: bincode::de::Decode<()>,
{
  bincode::decode_from_slice::<T, bincode::config::Configuration>(
    bytes,
    BINCODE_CONFIG,
  )
  .unwrap()
}
