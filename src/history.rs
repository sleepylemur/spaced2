use anyhow::{anyhow, Error};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, ErrorKind},
};

use crate::cards::Card;

pub fn load_history(filename: &str, cards: &mut HashMap<String, Card>) -> Result<(), Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\d+)\s+(\w+)\s([tf])$").unwrap();
    }
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => return Ok(()),
            _ => return Err(Into::into(err)),
        },
    };
    let mut lines = BufReader::new(file).lines();
    let mut last_tag = None;
    let mut history_num = 0;
    while let Some(Ok(line)) = lines.next() {
        match RE.captures(&line) {
            Some(captures) => {
                let timestamp: u64 = captures[1].parse()?;
                let tag = &captures[2];
                let correct = &captures[3] == "t";

                match cards.get_mut(tag) {
                    Some(card) => card.update(timestamp, correct, &last_tag, history_num),
                    None => return Err(anyhow!("Couldn't find card {}", tag)),
                };

                last_tag = Some(String::from(tag));
                history_num += 1;
            }
            None => return Err(anyhow!("failed to parse {}", line)),
        }
    }
    Ok(())
}
