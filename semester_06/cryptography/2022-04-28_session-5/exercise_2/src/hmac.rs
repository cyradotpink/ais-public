use sha1::{Digest, Sha1};

pub fn calculate_hmac(key: &[u8], message: &[u8]) -> [u8; 20] {
  let mut bs_key = [0; 64];
  if key.len() > 64 {
    let digest = Sha1::digest(key);
    for i in (0..digest.len()).take_while(|i| i < &64) {
      bs_key[i] = digest[i];
    }
  } else {
    for i in 0..key.len() {
      bs_key[i] = key[i];
    }
  }
  let mut outer_key = [0; 64];
  let mut inner_key = [0; 64];
  for i in 0..64 {
    outer_key[i] = bs_key[i] ^ 0x5c;
    inner_key[i] = bs_key[i] ^ 0x36;
  }
  let mut inner_hasher = Sha1::new();
  inner_hasher.update(&inner_key);
  inner_hasher.update(&message);
  let mut outer_hasher = Sha1::new();
  outer_hasher.update(&outer_key);
  outer_hasher.update(&inner_hasher.finalize());
  outer_hasher.finalize().into()
}
