use std::{collections::HashMap, env::Args};

pub fn parse_args(args: Args, spec: Vec<(&str, char)>) -> HashMap<String, String> {
  let args: Vec<String> = args.collect();

  let mut map = HashMap::new();

  for (name, short) in spec.iter() {
    if let Some(value) = args.iter().find_map(|arg| {
      if &arg[..2] == "--" {
        if let Some((found_name, value)) = arg[2..].split_once("=") {
          if found_name == *name {
            return Some(String::from(value));
          }
        }
      } else if &arg[..1] == "-" {
        if let Some(ch) = arg.chars().nth(1) {
          if ch == *short {
            return Some(String::from(&arg[2..]));
          }
        }
      }
      return None;
    }) {
      map.insert(String::from(*name), value);
    }
  }
  map
}
