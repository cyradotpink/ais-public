use hex;
use num::BigUint;
use rand;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::env;
use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

struct Generator {
  state: [u8; 89],
  count: u64,
  max_count: u64,
}

impl Generator {
  fn new(max_count: u64, linked_hash: &[u8; 32], initial_random_part: &[u8; 16]) -> Generator {
    let mut generator = Generator {
      state: [58; 89],
      count: 0,
      max_count: max_count,
    };
    let name = "SimonPaul";
    name
      .bytes()
      .enumerate()
      .map(|(i, val)| generator.state[i + 16 + 64] = val)
      .nth(9);
    for i in 0..16 {
      generator.state[i] = initial_random_part[i]
    }
    let mut hash_hex = [0; 64];
    hex::encode_to_slice(linked_hash, &mut hash_hex).unwrap();
    for i in 0..64 {
      generator.state[i + 16] = hash_hex[i];
    }

    generator
  }
  fn next<F>(&mut self, f: F) -> bool
  where
    F: Fn(&[u8; 89]),
  {
    if self.count >= self.max_count {
      return false;
    } else {
      f(&self.state);
    }
    self.count = self.count + 1;
    for i in 0..17 {
      if i == 16 {
        self.count = self.max_count;
      }
      if self.state[i] < 57
        || (self.state[i] > 64 && self.state[i] < 90)
        || (self.state[i] > 96 && self.state[i] < 122)
      {
        self.state[i] = self.state[i] + 1;
        break;
      } else if self.state[i] == 57 {
        self.state[i] = 65;
        break;
      } else if self.state[i] == 90 {
        self.state[i] = 97;
        break;
      } else {
        self.state[i] = 48;
      }
    }
    true
  }
}

// TODO: generator should be optional.
struct MainToHasher {
  hash_to_beat: [u8; 32],
  generator: Generator,
}

struct HashFoundInfo {
  hash: [u8; 32],
  name: String,
  random_part: [u8; 16],
}

struct HasherToMain {
  id: u8,
  target_was: [u8; 32],
  hash_found: Option<HashFoundInfo>,
}

struct BlockchainToMain {
  hash: [u8; 32],
  name: String,
  random_part: String,
}

struct MainToBlockchain {
  name: String,
  random_part: String,
}

enum AnyToMain {
  BlockchainToMain(BlockchainToMain),
  HasherToMain(HasherToMain),
}

fn index_to_code_point(index: usize) -> u8 {
  let index = index as u8;
  if index < 10 {
    return index + 48;
  } else if index < 36 {
    return index + 65 - 10;
  } else {
    return index + 97 - 26 - 10;
  }
}

fn is_smaller_hash(a: &[u8; 32], b: &[u8; 32]) -> bool {
  for i in 0..32 {
    if a[i] < b[i] {
      return true;
    } else if a[i] > b[i] {
      return false;
    }
  }
  return false;
}

fn make_random_part_state(int: u128, suffix: u32) -> Option<[u8; 16]> {
  let mut int = int;
  let mut suffix = suffix;
  let radix: u8 = 62;
  let mut out = [index_to_code_point(0); 16];
  for i in 0..6 {
    if suffix == 0 {
      break;
    }
    let x = suffix % (radix as u64).pow((i + 1) as u32) as u32;
    suffix = suffix - x;
    out[i + 10] = index_to_code_point((x / (radix as u32).pow(i as u32) as u32) as usize);
  }
  for i in 0..11 {
    if int == 0 {
      break;
    }
    if i >= (10) {
      return None;
    }
    let x = int % (radix as u64).pow((i + 1) as u32) as u128;
    int = int - x;
    out[i as usize] = index_to_code_point((x / (radix as u128).pow(i as u32) as u128) as usize);
  }

  Some(out)
}

// Iterator that yields candidate generators
struct GeneratorGenerator {
  next_job_begin: u128,
  job_gap: u64,
  linked_hash: [u8; 32],
  random_part_suffix: u32,
}
impl GeneratorGenerator {
  fn new(
    first_job_begin: u128,
    job_gap: u64,
    linked_hash: [u8; 32],
    random_part_suffix: u32,
  ) -> GeneratorGenerator {
    GeneratorGenerator {
      next_job_begin: first_job_begin,
      job_gap: job_gap,
      linked_hash: linked_hash,
      random_part_suffix: random_part_suffix,
    }
  }
  fn next(&mut self) -> Option<Generator> {
    if let Some(random_part_state) =
      make_random_part_state(self.next_job_begin, self.random_part_suffix)
    {
      let generator = Generator::new(self.job_gap, &self.linked_hash, &random_part_state);
      self.next_job_begin = self.next_job_begin + self.job_gap as u128;
      return Some(generator);
    } else {
      return None;
    }
  }
}

fn divide_hash(hash: &[u8; 32], factor: u32) -> [u8; 32] {
  let vec = (BigUint::from_bytes_be(hash) / factor).to_bytes_be();
  let len = vec.len();
  let mut out: [u8; 32] = [0; 32];
  vec
    .iter()
    .enumerate()
    .map(|(i, val)| out[32 - len + i] = *val)
    .nth(32);
  out
}

fn hasher_worker(id: u8, tx: mpsc::Sender<AnyToMain>, rx: mpsc::Receiver<MainToHasher>) {
  // For performance, only sometimes check if the main thread has sent an updated target
  let nexts_before_work_check = 10_000_000;
  // Initialise everything with zeroes to make the compiler happy
  let mut generator = Generator::new(0, &[0; 32], &[0; 16]);
  let mut hash_to_beat = [0; 32];
  let mut out_of_work = true; // Initially, wait for work to arrive
  loop {
    // If the current iterator has been consumed fully, block the thread until new work is available
    if out_of_work {
      let message = rx.recv().unwrap();
      generator = message.generator;
      hash_to_beat = message.hash_to_beat;
    // Else, check if the work target has changed
    } else if let Ok(message) = rx.try_recv() {
      generator = message.generator;
      hash_to_beat = message.hash_to_beat;
    }
    for _ in 0..nexts_before_work_check {
      out_of_work = !generator.next(|candidate| {
        let hash: [u8; 32] = Sha256::digest(candidate).try_into().unwrap();
        /*
        println!("beating {:?}", &hash_to_beat);
        println!(
          "Hash: {} with random part {}",
          hex::encode(hash),
          str::from_utf8(&candidate[0..16]).unwrap()
        );
         */
        if is_smaller_hash(&hash, &hash_to_beat) {
          let mut random_part = [0; 16];
          for i in 0..16 {
            random_part[i] = candidate[i];
          }
          tx.send(AnyToMain::HasherToMain(HasherToMain {
            hash_found: Some(HashFoundInfo {
              hash,
              name: String::from("SimonPaul"),
              random_part,
            }),
            id,
            target_was: hash_to_beat,
          }))
          .unwrap();
        }
      });
      if out_of_work {
        tx.send(AnyToMain::HasherToMain(HasherToMain {
          hash_found: None,
          id,
          target_was: hash_to_beat,
        }))
        .unwrap();
        break;
      }
    }
  }
}

struct LineReader<T> {
  readable: T,
  out_queue: String,
  current_line: String,
}

impl<T> LineReader<T>
where
  T: Read,
{
  fn new(readable: T) -> LineReader<T> {
    LineReader {
      readable,
      out_queue: String::new(),
      current_line: String::new(),
    }
  }
}

impl<T> Iterator for LineReader<T>
where
  T: Read,
{
  type Item = String;

  fn next(&mut self) -> Option<String> {
    let mut outbuf = [0; 1024];
    loop {
      let mut should_break = false;
      for _ in 0..self.out_queue.chars().count() {
        let char = self.out_queue.remove(0);
        if char != '\n' {
          self.current_line.push(char);
        } else {
          should_break = true;
          break;
        }
      }
      if should_break {
        break;
      }
      match self.readable.read(&mut outbuf) {
        Ok(n) => {
          self
            .out_queue
            .push_str(str::from_utf8(&outbuf[..n]).unwrap_or(""));
        }
        Err(_) => {
          return None;
        }
      }
    }
    let line = self.current_line.clone();
    self.current_line.clear();
    Some(line)
  }
}

fn try_interpret_line(line: String) -> Option<BlockchainToMain> {
  println!("New line: {}", line);
  if line.starts_with("_") {
    return None;
  }
  let mut split = line.split(' ');
  let random_part = split.next()?.to_owned();
  let name = split.next()?.to_owned();
  let hash_hex = split.next()?;
  let mut hash = [0; 32];
  hex::decode_to_slice(hash_hex, &mut hash).ok()?;
  Some(BlockchainToMain {
    random_part,
    name,
    hash,
  })
}

fn blockchain_interactor(tx: mpsc::Sender<AnyToMain>, rx: mpsc::Receiver<MainToBlockchain>) {
  let pwd = env::current_dir().unwrap();
  let js_path = pwd.ancestors().nth(2).unwrap().join("http_stuff/index.js");
  let mut node_child = Command::new("node")
    .arg(js_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Node process failed to be created");

  let _ = node_child
    .stderr
    .take()
    .expect("Failed to take node stderr");

  let stdout = node_child.stdout.take().expect("Failed to take stdout");

  let line_reader = LineReader::new(stdout);

  thread::spawn(move || {
    for line in line_reader {
      if let Some(info) = try_interpret_line(line) {
        tx.send(AnyToMain::BlockchainToMain(info)).unwrap();
      }
    }
  });

  let mut stdin = node_child.stdin.take().expect("Failed to take node stdin");

  thread::spawn(move || {
    for recv in rx {
      let write_str = format!("{} {}", recv.random_part, recv.name);
      stdin.write(&write_str.as_bytes()).unwrap();
    }
  });
}

fn main() {
  let num_of_hashers: u8 = 16;
  let job_gap = 100_000_000;
  let perf_print = false;

  let mut linked_hash: [u8; 32] = [0; 32];
  let mut hash_to_beat = [0; 32]; //divide_hash(&linked_hash, 2);

  let mut hasher_senders = Vec::new();

  let (to_main_tx, main_rx) = mpsc::channel();

  let mut gen_gen = GeneratorGenerator::new(0, job_gap, linked_hash, rand::random());

  let mut estimated_tested_count = 0;
  let begin_time = Instant::now();

  for i in 0..num_of_hashers {
    let worker_tx_clone = to_main_tx.clone();
    let (main_tx, worker_rx) = mpsc::channel();
    thread::spawn(move || {
      hasher_worker(i, worker_tx_clone, worker_rx);
    });
    hasher_senders.push(main_tx);
  }

  let from_blockchain_tx_clone = to_main_tx.clone();
  let (to_blockchain_tx, blockchain_rx) = mpsc::channel();
  thread::spawn(move || {
    blockchain_interactor(from_blockchain_tx_clone, blockchain_rx);
  });

  for message in main_rx.into_iter() {
    match message {
      AnyToMain::BlockchainToMain(message) => {
        linked_hash = message.hash;
        gen_gen = GeneratorGenerator::new(0, job_gap, linked_hash, rand::random());
        hash_to_beat = divide_hash(&message.hash, 2);
        println!("Now targeting {}; {}", message.name, message.random_part);
        for tx in hasher_senders.iter() {
          tx.send(MainToHasher {
            hash_to_beat: hash_to_beat.clone(),
            generator: gen_gen.next().unwrap(), // Panics when literally every possible random part was tried. I will be dead by the time this happens.
          })
          .unwrap();
        }
      }
      AnyToMain::HasherToMain(message) => {
        if let HasherToMain {
          hash_found: Some(hash_found),
          id: _,
          target_was,
        } = message
        {
          if target_was == hash_to_beat {
            println!("Found smaller hash");
            println!(
              "Target was {}, found {}",
              hex::encode(target_was),
              hex::encode(hash_found.hash)
            );
            println!(
              "with random part {} and name {}",
              str::from_utf8(&hash_found.random_part).unwrap(),
              hash_found.name
            );
            to_blockchain_tx
              .send(MainToBlockchain {
                name: hash_found.name,
                random_part: str::from_utf8(&hash_found.random_part).unwrap().to_owned(),
              })
              .unwrap();
            hash_to_beat = hash_found.hash;
            for tx in hasher_senders.iter() {
              tx.send(MainToHasher {
                hash_to_beat: hash_to_beat.clone(),
                generator: gen_gen.next().unwrap(), // Panics when literally every possible random part was tried. I will be dead by the time this happens.
              })
              .unwrap();
            }
            // std::process::exit(0);
          }
        }
        if perf_print {
          estimated_tested_count = estimated_tested_count + job_gap;
          if perf_print {
            println!(
              "Estimated performance is {:?} hashes per second",
              (estimated_tested_count as u128
                / (Instant::now().duration_since(begin_time).as_millis() + 1))
                * 1000
            );
          }
          // println!("Job completed by ID {}", message.id);
          // println!("Next job starts at {}", gen_gen.next_job_begin);
        }
        hasher_senders[message.id as usize]
          .send(MainToHasher {
            hash_to_beat: hash_to_beat.clone(),
            generator: gen_gen.next().unwrap(),
          })
          .unwrap();
      }
    }
  }
}
