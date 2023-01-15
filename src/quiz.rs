use std::{
    collections::HashMap,
    io::{Stdin, Stdout},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Error;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};

use crate::{
    card_selection::{get_possible, random_card},
    cards::{self, activate_cards, Card},
    history::History,
};

fn current_timestamp() -> Result<u64, Error> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .map_err(anyhow::Error::msg)
}

pub fn quiz(filename: &str, stdout: &mut Stdout, stdin: &Stdin) -> Result<(), Error> {
    println!("{}", filename);

    let mut cards: HashMap<String, Card> = cards::Card::from_file(&format!("cards/{}", filename))?
        .into_iter()
        .map(|card| (card.tag.clone(), card))
        .collect();

    let mut history = History::open(&format!("history/{}", filename))?;
    history.parse(&mut cards)?;

    let mut answer = String::new();
    let mut last_tag: Option<String> = None;
    let mut last_correct = None;
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    loop {
        let (possible, inactive_count, to_review_count) = get_possible(&mut cards, &last_tag);
        if to_review_count == 0 {
            if inactive_count == 0 {
                println!("All cards reviewed.\npress anything to exit");
                stdin.read_line(&mut answer)?;
                break;
            } else {
                println!(
                    "All cards reviewed, {} unlearned.\na to add. anything else to quit.",
                    inactive_count
                );
                stdin.read_line(&mut answer)?;
                if answer.trim() == "a" {
                    activate_cards(&mut cards, 5);
                } else {
                    break;
                }
            }
        } else if let Some(card) = random_card(&possible, &mut cards) {
            match last_correct {
                Some(true) => println!("correct"),
                Some(false) => println!("wrong"),
                None => (),
            }
            println!(
                "reviewing {}, unlearned {}",
                to_review_count, inactive_count
            );
            println!("{}", card.question);
            stdin.read_line(&mut answer)?;
            if answer.len() == 0 {
                break;
            }

            let is_correct = if answer.trim() == card.answer {
                println!("correct!");
                true
            } else {
                println!("nope");
                false
            };

            card.update(current_timestamp()?, is_correct, &last_tag, history.num);
            history.persist_update(card, is_correct)?;

            last_tag = Some(card.tag.clone());
            last_correct = Some(is_correct);
        } else {
            break;
        }
        answer.truncate(0);
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    }
    Ok(())
}
