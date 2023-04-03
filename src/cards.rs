use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
    iter::Peekable,
    path::Path,
};

#[derive(Debug)]
pub enum CardType {
    Text,
    Keys,
}

#[derive(Debug)]
pub struct Card {
    pub card_type: CardType,
    pub tag: String,
    pub question: String,
    pub answer: String,
    pub num_correct: u32,
    pub last_followed: Option<String>,
    pub last_num: u32,
    pub last_ts: u64,
    pub expected_retention_days: u16,
    pub active: bool,
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
    pub fn from_file(path: &Path) -> Result<Vec<Card>, Error> {
        let mut cards = vec![];
        let file = File::open(path)?;
        let mut lines = BufReader::new(file).lines().peekable();
        skip_blank(&mut lines);
        while !lines.peek().is_none() {
            let tag_line = lines.next().unwrap().unwrap();
            let mut tag_line_words = tag_line.split_whitespace();

            cards.push(Card {
                tag: tag_line_words.next().unwrap().to_string(),
                card_type: match tag_line_words.next().unwrap_or_default() {
                    "keys" => CardType::Keys,
                    _ => CardType::Text,
                },
                question: lines.next().unwrap().unwrap(),
                answer: lines.next().unwrap().unwrap(),
                num_correct: 0,
                last_followed: None,
                last_num: 0,
                last_ts: 0,
                expected_retention_days: 0,
                active: false,
            });
            skip_blank(&mut lines);
        }
        Ok(cards)
    }

    pub fn update(&mut self, ts: u64, correct: bool, followed: &Option<String>, history_num: u32) {
        if correct {
            self.num_correct += 1;
            if self.num_correct >= 3 {
                if self.expected_retention_days == 0 {
                    self.expected_retention_days = 1;
                } else {
                    self.expected_retention_days *= 2;
                }
            }
        } else {
            self.num_correct = 0;
            self.expected_retention_days /= 2;
        }

        self.last_num = history_num;
        self.last_ts = ts;
        self.last_followed = followed.as_deref().map(|tag| String::from(tag));
        self.active = true;
    }
}
