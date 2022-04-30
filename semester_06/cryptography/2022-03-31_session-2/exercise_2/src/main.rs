use hex;
use sha1::{Digest, Sha1};
use std::convert::TryInto;
use std::str;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

enum FromWorkerMessageContent {
    Hash((String, String)),
    Done,
}

struct FromWorkerMessage {
    id: u8,
    content: FromWorkerMessageContent,
}

impl FromWorkerMessage {
    fn done(id: u8) -> FromWorkerMessage {
        FromWorkerMessage {
            id: id,
            content: FromWorkerMessageContent::Done,
        }
    }
    fn hash(id: u8, plain: String, hash: String) -> FromWorkerMessage {
        FromWorkerMessage {
            id: id,
            content: FromWorkerMessageContent::Hash((plain, hash)),
        }
    }
}

struct RandomPassword {
    state: [u8; 32],
    state_size: usize,
    count: u64,
    max_count: u64,
}

impl RandomPassword {
    fn new(max_count: u64, initial_state: Vec<u8>) -> RandomPassword {
        let state_size = initial_state.len();
        let mut initial = [0; 32];
        initial_state
            .iter()
            .enumerate()
            .map(|(i, val)| initial[i] = *val)
            .nth(state_size - 1);

        RandomPassword {
            state: initial,
            state_size: state_size,
            count: 0,
            max_count: max_count,
        }
    }
    fn next<F>(&mut self, f: F) -> bool
    where
        F: Fn(Option<&[u8; 32]>),
    {
        if self.count >= self.max_count {
            f(None);
            return false;
        } else {
            f(Some(&self.state));
        }
        self.count = self.count + 1;
        for i in 0..self.state_size + 1 {
            if i == self.state_size {
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

fn hasher_worker(
    id: u8,
    hashmap: [([u8; 5], [u8; 20]); 256],
    tx: mpsc::Sender<FromWorkerMessage>,
    rx: mpsc::Receiver<RandomPassword>,
) {
    tx.send(FromWorkerMessage::done(id)).unwrap();
    for iterator in rx {
        let mut cont = true;
        let state_size = iterator.state_size;
        let mut iterator = iterator;
        while cont {
            cont = iterator.next(|state| match state {
                Some(candidate) => {
                    let hash: [u8; 20] = Sha1::digest(&candidate[..state_size]).try_into().unwrap();
                    let (name, found_hash) = hashmap[hash[0] as usize];
                    // println!("{:?}", &candidate[..state_size]);
                    if name[0] != 32 && found_hash == hash {
                        tx.send(FromWorkerMessage::hash(
                            id,
                            str::from_utf8(candidate).unwrap().to_owned(),
                            str::from_utf8(&name).unwrap().to_owned(),
                        ))
                        .unwrap();
                    }
                    // println!("{:?}", hash);
                }
                None => (),
            });
        }
        /*
        for candidate in iterator {
            // println!("{}", candidate);
            let hash: [u8; 20] = Sha1::digest(&candidate).try_into().unwrap();
            let (name, found_hash) = hashmap[hash[0] as usize];
            // println!("{:?}", name[0]);
            if name[0] != 32 && found_hash == hash {
                tx.send(FromWorkerMessage::hash(
                    id,
                    candidate,
                    str::from_utf8(&name).unwrap().to_owned(),
                ))
                .unwrap();
            }
        }
                */
        tx.send(FromWorkerMessage::done(id)).unwrap();
    }
}

fn get_hashmap() -> [([u8; 5], [u8; 20]); 256] {
    let arr = [
        ("joe", "06a12c67249567c66725263ac26d5f508448f1e1"),
        ("bob", "079f8191fe2fc4b01bb6415083db2ed481b7ec32"),
        ("ute", "db23fe065e9f857e4cd3398a25299be0bc383c2b"),
        ("paul", "4a660a7d88dbde0b75dd2f6399e23226c259b7ff"),
        ("nina", "e01a18a0d1b0dbe455c56de57079f52015554f68"),
        ("anja", "9fa154f3a0baa0aadf70066f1f4dbd62258b1c99"),
        ("fritz", "ff5cf374186912339aaa14b73e90f1545d43aa96"),
        ("peter", "fd7e698d04cad5a2a20a9256cbf929aee58732e9"),
    ];
    let mut fake_hashmap: [([u8; 5], [u8; 20]); 256] = [([32; 5], [0; 20]); 256];
    for (name, hash) in arr {
        let mut bytes = [0_u8; 20];
        hex::decode_to_slice(hash, &mut bytes).unwrap();
        name.bytes()
            .enumerate()
            .map(|(i, byte)| fake_hashmap[bytes[0] as usize].0[i] = byte)
            .nth(4);
        fake_hashmap[bytes[0] as usize].1 = bytes;
    }
    // println!("{:?}", fake_hashmap);
    //hashmap
    fake_hashmap
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

fn make_iterator_state(int: u64, size: usize) -> Option<Vec<u8>> {
    let mut int = int;
    let radix: u8 = 62;
    let mut out = vec![index_to_code_point(0); size];
    for i in 0..radix {
        if int == 0 {
            break;
        }
        if i >= (size as u8) {
            return None;
        }
        let x = int % (radix as u64).pow((i + 1) as u32) as u64;
        int = int - x;
        out[i as usize] = index_to_code_point((x / (radix as u64).pow(i as u32) as u64) as usize)
    }

    Some(out)
}

fn main() {
    let num_of_workers: u8 = 16;
    let mut state_size = 2;
    let state_gap = 100_000_000;
    let perf_print = true;

    let hashmap = get_hashmap();
    let mut worker_handles = Vec::new();
    let mut worker_senders = Vec::new();

    let (worker_tx, main_rx) = mpsc::channel();

    for i in 0..num_of_workers {
        let worker_tx_clone = worker_tx.clone();
        let (main_tx, worker_rx) = mpsc::channel();
        worker_senders.push(main_tx);
        let hashmap = hashmap.clone();

        let handle = thread::spawn(move || {
            hasher_worker(i, hashmap, worker_tx_clone, worker_rx);
        });

        worker_handles.push(handle);
    }

    let mut rx_iter = main_rx.into_iter();
    let cont = true;

    let mut state_int = 0;
    let mut state_size = 2;
    let state_gap = 10_000_000;

    let mut estimated_tested_count: u64 = 0;

    let begin_time = Instant::now();

    while cont {
        let recv = rx_iter.next().unwrap();
        match recv.content {
            FromWorkerMessageContent::Hash((candidate, name)) => {
                println!("Found password for {}: {}", name, candidate);
            }
            FromWorkerMessageContent::Done => {
                estimated_tested_count = estimated_tested_count + state_gap;
                println!(
                    "Estimated performance is {:?} hashes per second",
                    (estimated_tested_count as u128
                        / (Instant::now().duration_since(begin_time).as_millis() + 1))
                        * 1000
                );
                let begin_state;
                loop {
                    if let Some(state) = make_iterator_state(state_int, state_size) {
                        state_int = state_int + state_gap;
                        begin_state = state;
                        break;
                    } else {
                        state_int = 0;
                        state_size = state_size + 1;
                    }
                }
                /*println!(
                    "Supplying new work to worker thread with ID {}; Beginning candidate is {:?}",
                    recv.id, begin_state
                );*/
                worker_senders[recv.id as usize]
                    .send(RandomPassword::new(state_gap, begin_state))
                    .unwrap();
            }
        }
    }

    for handle in worker_handles.into_iter() {
        handle.join().unwrap()
    }
}
