use sha2::digest::Digest;

pub fn hash<D: Digest>(key: &str, salt: &str, output: &mut [u8]) {
  let mut hasher = D::new();
  hasher.update(key.as_bytes());
  hasher.update(b"$");
  hasher.update(salt.as_bytes());
  output.copy_from_slice(hasher.finalize().as_slice());
}
