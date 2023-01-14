use anyhow::{anyhow, Error};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use crate::cards::Card;

pub struct History {
    file: File,
    pub num: u32,
}

impl History {
    pub fn open(filename: &str) -> Result<History, Error> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(filename)?;
        Ok(History { file, num: 0 })
    }

    pub fn parse(&mut self, cards: &mut HashMap<String, Card>) -> Result<(), Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+)\s+(\w+)\s([tf])$").unwrap();
        }
        let mut lines = BufReader::new(&self.file).lines();
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
        self.num = history_num;
        Ok(())
    }

    pub fn persist_update(&mut self, card: &Card, is_correct: bool) -> Result<(), Error> {
        writeln!(
            self.file,
            "{} {} {}",
            card.last_ts,
            card.tag,
            if is_correct { 't' } else { 'f' }
        )?;
        self.num += 1;
        Ok(())
    }
}
