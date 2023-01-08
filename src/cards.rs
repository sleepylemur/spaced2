use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
    iter::Peekable,
};

#[derive(Debug)]
pub struct Card {
    pub tag: String,
    pub question: String,
    pub answer: String,
    pub num_correct: u32,
    pub last_followed: Option<String>,
    pub last_num: u32,
    pub last_ts: u64,
    pub expected_retention_days: u16,
}

fn skip_blank<I>(lines: &mut Peekable<I>)
where
    I: Iterator<Item = Result<String, Error>>,
{
    while let Some(result) = lines.peek() {
        if let Ok(line) = result {
            if line.len() == 0 {
                lines.next();
                continue;
            }
        }
        break;
    }
}

impl Card {
    pub fn from_file(filename: &str) -> Result<Vec<Card>, Error> {
        let mut cards = vec![];
        let file = File::open(filename)?;
        let mut lines = BufReader::new(file).lines().peekable();
        skip_blank(&mut lines);
        while !lines.peek().is_none() {
            cards.push(Card {
                tag: lines.next().unwrap().unwrap(),
                question: lines.next().unwrap().unwrap(),
                answer: lines.next().unwrap().unwrap(),
                num_correct: 0,
                last_followed: None,
                last_num: 0,
                last_ts: 0,
                expected_retention_days: 0,
            });
            skip_blank(&mut lines);
        }
        Ok(cards)
    }
}
