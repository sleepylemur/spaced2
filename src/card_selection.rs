use rand::Rng;
use std::{
    collections::HashMap,
    ops::Deref,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::cards::Card;

pub fn get_possible<'a>(
    cards: &HashMap<String, Card>,
    last_tag: &Option<String>,
) -> (Vec<String>, usize, usize) {
    let mut possible = vec![];
    let mut possible_repeats = vec![];
    let mut inactive = 0;
    let now = SystemTime::now();
    for (_, card) in cards {
        let mut is_repeat = false;
        if !card.active {
            inactive += 1;
            continue;
        }
        if let Some(tag) = last_tag {
            if card.tag.deref() == tag || card.last_followed == *last_tag {
                is_repeat = true;
            }
        }
        if card.num_correct < 3 {
            if is_repeat {
                possible_repeats.push(card.tag.clone());
            } else {
                possible.push(card.tag.clone());
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
                    possible.push(card.tag.clone());
                }
            }
        }
    }
    let to_review_count = possible.len() + possible_repeats.len();
    (
        if possible.len() == 0 {
            possible_repeats
        } else {
            possible
        },
        inactive,
        to_review_count,
    )
}

pub fn random_tag(possible: &Vec<String>) -> Option<String> {
    if possible.is_empty() {
        None
    } else {
        let i = rand::thread_rng().gen_range(0..possible.len());
        Some(possible[i].clone())
    }
}

pub fn random_card<'a>(
    possible: &Vec<String>,
    cards: &'a mut HashMap<String, Card>,
) -> Option<&'a mut Card> {
    if possible.len() == 0 {
        None
    } else {
        let i = rand::thread_rng().gen_range(0..possible.len());
        cards.get_mut(&possible[i])
    }
}
