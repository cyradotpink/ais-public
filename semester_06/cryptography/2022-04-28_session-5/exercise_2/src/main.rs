mod args;
mod hmac;
mod hotp;

use std::{
  env,
  error::Error,
  fmt::Display,
  io::{self, BufRead},
};

/*
Configure your YubiKey as follows to test against the default configuration of this program in validator mode:
ykpersonalize -1 -a0000000000000000000000000000000000000000 -ooath-hotp -ooath-imf=0 -ofixed=cccccccc -oserial-api-visible -oappend-cr
              ^ Target the key's first slot                 ^ Use HOTP  ^ Initialise moving factor to 0                    ^ Append a carriage return
                 ^ Use a secret key of 20 zero-bytes
*/

/*
The default configuration is equivalent to running this program with command line arguments as follows:
./exercise_2 -mkey -c0 -k0000000000000000000000000000000000000000 -l6 -w20
             ^ Use the program as a key simulator                 ^ Use 6-digit HOTP values
                    ^ Initialise moving factor to 0                   ^ Use a window size of 20
                       ^ Use a secret key of 20 zero-bytes
*/

// The program mode
const MODE: SimulateMode = SimulateMode::Key;
// The initial value for the HOTP validator's moving factor
const COUNTER_INIT: u64 = 0;
// The secret key for the validator to HMAC the counter with
const SECRET_KEY: &str = "0000000000000000000000000000000000000000";
// Number of digits in HOTP values (configure with "-l")
const OTP_LENGTH: usize = 6;
// Number of HOTP values ahead of the counter that the validator considers
const VALIDATOR_WINDOW_SIZE: usize = 20;

fn isolate_otp(input: &str, otp_length: usize) -> Option<&str> {
  let input = input.trim_end();
  let char_count = input.chars().count();
  if char_count < otp_length {
    return None;
  }
  let substr = &input[char_count - otp_length..];
  if substr.chars().all(|ch| ch.is_ascii_digit()) {
    Some(substr)
  } else {
    None
  }
}

enum SimulateMode {
  Key,
  Validator,
}

struct RunArgs {
  simulate_mode: SimulateMode,
  counter_init: u64,
  key: Vec<u8>,
  otp_length: usize,
  window_size: usize,
}

#[derive(Debug)]
enum ArgParseError {
  Mode,
  CounterInit,
  SecretKey,
  OtpLength,
  ValidatorWindowSize,
}

impl Display for ArgParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Failed to parse command line argument {:?}", self)
  }
}
impl Error for ArgParseError {}

fn parse_args() -> Result<RunArgs, ArgParseError> {
  let mut args = RunArgs {
    simulate_mode: MODE,
    counter_init: COUNTER_INIT,
    key: hex::decode(SECRET_KEY).unwrap(),
    otp_length: OTP_LENGTH,
    window_size: VALIDATOR_WINDOW_SIZE,
  };

  let parsed = args::parse_args(
    env::args(),
    vec![
      ("mode", 'm'),
      ("counter", 'c'),
      ("key", 'k'),
      ("length", 'l'),
      ("window", 'w'),
    ],
  );
  for (k, v) in parsed.iter() {
    let v = &v[..];
    match &k[..] {
      "mode" => {
        args.simulate_mode = match v {
          "key" => Ok(SimulateMode::Key),
          "validator" => Ok(SimulateMode::Validator),
          _ => Err(ArgParseError::Mode),
        }?
      }
      "counter" => args.counter_init = v.parse().map_err(|_| ArgParseError::CounterInit)?,
      "key" => args.key = hex::decode(v).map_err(|_| ArgParseError::SecretKey)?,
      "length" => args.otp_length = v.parse().map_err(|_| ArgParseError::OtpLength)?,
      "window" => args.window_size = v.parse().map_err(|_| ArgParseError::ValidatorWindowSize)?,
      _ => {}
    }
  }

  Ok(args)
}

fn run_validator(run_args: &RunArgs) -> Result<(), Box<dyn Error>> {
  let mut validator = hotp::OtpValidator::new(
    run_args.counter_init,
    &run_args.key,
    run_args.otp_length,
    run_args.window_size,
  );

  let mut buffer = String::new();
  let stdin = io::stdin();
  let mut handle = stdin.lock();

  print!("Executing as HOTP validator. Start providing HOTP values, for example by using your YubiKey.\n> ");
  io::Write::flush(&mut io::stdout())?;
  buffer.clear();
  while handle.read_line(&mut buffer)? > 0 {
    if let Some(otp) = isolate_otp(&buffer, run_args.otp_length) {
      let result = validator.validate(otp);
      match result {
        Ok(counter) => {
          println!("\"{otp}\" is valid at counter value {}", counter);
        }
        Err(err) => {
          println!("\"{otp}\" is invalid ({:?})", &err);
        }
      }
    } else {
      println!("Ignoring malformed input");
    }
    print!("> ");
    io::Write::flush(&mut io::stdout())?;
    buffer.clear();
  }
  Ok(())
}

fn run_key(run_args: &RunArgs) -> Result<(), Box<dyn Error>> {
  let mut generator =
    hotp::OtpGenerator::new(run_args.counter_init, &run_args.key, run_args.otp_length);

  let mut buffer = String::new();
  let stdin = io::stdin();
  let mut handle = stdin.lock();

  println!("Executing as HOTP generator. Enter a newline to start generating values.");
  io::Write::flush(&mut io::stdout())?;
  while handle.read_line(&mut buffer)? > 0 {
    if let Some(otp) = generator.next() {
      print!(
        "HOTP value \"{otp}\" at counter value {}",
        generator.counter_state() - 1
      );
      io::Write::flush(&mut io::stdout())?;
    } else {
      break;
    }
  }
  Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
  let run_args = parse_args().map_err(|err| {
    println!("{}", err);
    err
  })?;
  (match run_args.simulate_mode {
    SimulateMode::Key => run_key,
    SimulateMode::Validator => run_validator,
  })(&run_args)?;
  Ok(())
}
