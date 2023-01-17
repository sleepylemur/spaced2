use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Error};
use rand::Rng;

use crate::{cards::Card, history::History};

pub struct Deck {
    current_tag: Option<String>,
    last_tag: Option<String>,
    cards: HashMap<String, Card>,
    history: History,
    possible: Vec<String>,
    pub inactive: usize,
    pub reviewing: usize,
}

impl Deck {
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn load(filename: &str) -> Result<Deck, Error> {
        let mut cards: HashMap<String, Card> = Card::from_file(&format!("cards/{}", filename))?
            .into_iter()
            .map(|card| (card.tag.clone(), card))
            .collect();

        let mut history = History::open(&format!("history/{}", filename))?;
        history.parse(&mut cards)?;
        Ok(Deck {
            current_tag: None,
            last_tag: None,
            cards,
            history,
            possible: vec![],
            inactive: 0,
            reviewing: 0,
        })
    }

    pub fn update_possible(&mut self) {
        self.possible.truncate(0);
        let mut possible_repeats = vec![];
        self.inactive = 0;
        let now = SystemTime::now();
        for (_, card) in &self.cards {
            let mut is_repeat = false;
            if !card.active {
                self.inactive += 1;
                continue;
            }
            if let Some(tag) = &self.last_tag {
                if card.tag == *tag || card.last_followed == self.last_tag {
                    is_repeat = true;
                }
            }
            if card.num_correct < 3 {
                if is_repeat {
                    possible_repeats.push(card.tag.clone());
                } else {
                    self.possible.push(card.tag.clone());
                }
            } else {
                let hours_passed = now
                    .duration_since(UNIX_EPOCH + Duration::from_millis(card.last_ts))
                    .unwrap()
                    .as_secs()
                    / 60
                    / 60;
                if u64::from(card.expected_retention_days * 24 - 12) < hours_passed {
                    if is_repeat {
                        possible_repeats.push(card.tag.clone());
                    } else {
                        self.possible.push(card.tag.clone());
                    }
                }
            }
        }
        self.reviewing = self.possible.len() + possible_repeats.len();
        if self.possible.len() == 0 {
            self.possible = possible_repeats
        };
    }

    pub fn activate_cards(&mut self, num: u8) {
        let mut activated = 0;
        for (_, card) in self.cards.iter_mut() {
            if !card.active {
                card.active = true;
                activated += 1;
                if activated >= num {
                    break;
                }
            }
        }
    }

    pub fn current_card(&self) -> Option<&Card> {
        match &self.current_tag {
            None => None,
            Some(tag) => match self.cards.get(tag) {
                None => None,
                Some(card) => Some(&card),
            },
        }
    }

    pub fn next(&mut self) {
        if self.possible.is_empty() {
            self.current_tag = None;
            self.last_tag = None;
        } else {
            let i = rand::thread_rng().gen_range(0..self.possible.len());
            self.last_tag = self.current_tag.clone();
            self.current_tag = Some(self.possible[i].clone())
        }
    }

    pub fn answer(&mut self, correct: bool) -> Result<(), Error> {
        if let Some(tag) = &self.current_tag {
            match self.cards.get_mut(tag) {
                None => Err(anyhow!("unable to update card {:?}", self.current_tag)),
                Some(card) => {
                    card.update(
                        current_timestamp()?,
                        correct,
                        &self.last_tag,
                        self.history.num,
                    );
                    self.history.persist_update(card, correct)?;
                    Ok(())
                }
            }
        } else {
            Err(anyhow!("unable to update card {:?}", self.current_tag))
        }
    }
}

pub fn current_timestamp() -> Result<u64, Error> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .map_err(anyhow::Error::msg)
}
