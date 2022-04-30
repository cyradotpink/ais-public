use crate::hmac;

fn serialize_counter(counter: u64) -> [u8; 8] {
  let mut int = counter as u128;
  let mut out = [0; 8];
  for i in 0..8 {
    if int == 0 {
      break;
    }
    let x = int % (256 as u128).pow((i + 1) as u32) as u128;
    int = int - x;
    out[7 - i as usize] = (x / (256 as u128).pow(i as u32) as u128) as u8;
  }
  out
}

pub fn get_otp(counter: u64, key: &[u8], otp_length: usize) -> String {
  let counter = serialize_counter(counter);
  let mut hash = hmac::calculate_hmac(key, &counter);
  let offset = hash[19] as usize & 0x0f;
  hash[offset] = hash[offset] & 0x7f;
  let mut value = 0;
  for i in 0..4 {
    value = value + (hash[i + offset] as u64) * (256 as u64).pow(3 - i as u32);
  }
  let otp = format!("{}", value % (10 as u64).pow(otp_length as u32));
  String::from_iter(std::iter::repeat('0').take(otp_length - otp.chars().count())) + &otp
}

#[derive(Debug)]
pub struct OtpGenerator<'a> {
  counter: u64,
  key: &'a [u8],
  otp_length: usize,
}

impl<'a> OtpGenerator<'_> {
  pub fn new(counter_init: u64, key: &'a [u8], otp_length: usize) -> OtpGenerator<'a> {
    OtpGenerator {
      counter: counter_init,
      key,
      otp_length,
    }
  }
  pub fn counter_state(&self) -> u64 {
    self.counter
  }
}

impl Iterator for OtpGenerator<'_> {
  type Item = String;

  fn next(&mut self) -> Option<Self::Item> {
    if self.counter < u64::MAX {
      self.counter += 1;
      Some(get_otp(self.counter - 1, self.key, self.otp_length))
    } else {
      None
    }
  }
}

#[derive(Debug)]
pub enum OtpValidationError {
  MalformedCandidate,
  ReachedWindowLimit,
}

#[derive(Debug)]
pub struct OtpValidator<'a> {
  otp_generator: OtpGenerator<'a>,
  window_size: usize,
}

impl<'a> OtpValidator<'_> {
  pub fn new(
    counter_init: u64,
    key: &'a [u8],
    otp_length: usize,
    window_size: usize,
  ) -> OtpValidator {
    OtpValidator {
      otp_generator: OtpGenerator::new(counter_init, key, otp_length),
      window_size,
    }
  }

  pub fn validate(&mut self, candidate: &str) -> Result<u64, OtpValidationError> {
    if self.otp_generator.otp_length != candidate.chars().count()
      || candidate.chars().any(|v| !v.is_ascii_digit())
    {
      return Err(OtpValidationError::MalformedCandidate);
    }
    let counter_start = self.otp_generator.counter;
    let candidate_matched = self
      .otp_generator
      .by_ref()
      .take(self.window_size)
      .any(|value| candidate == value);
    if candidate_matched {
      Ok(self.otp_generator.counter - 1)
    } else {
      self.otp_generator.counter = counter_start;
      Err(OtpValidationError::ReachedWindowLimit)
    }
  }
}
