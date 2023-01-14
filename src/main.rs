use anyhow::Error;
use cards::Card;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use history::History;
use std::{
    collections::HashMap,
    io::{self},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::card_selection::{get_possible, random_card};

mod card_selection;
mod cards;
mod history;

fn current_timestamp() -> Result<u64, Error> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .map_err(anyhow::Error::msg)
}

fn activate_cards(cards: &mut HashMap<String, Card>, num: u8) {
    let mut activated = 0;
    for (_, card) in cards.iter_mut() {
        if !card.active {
            card.active = true;
            activated += 1;
            if activated >= num {
                break;
            }
        }
    }
}

fn main() -> Result<(), Error> {
    let mut cards: HashMap<String, Card> = cards::Card::from_file("cards/ex.txt")?
        .into_iter()
        .map(|card| (card.tag.clone(), card))
        .collect();

    let mut history = History::open("history")?;
    history.parse(&mut cards)?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut answer = String::new();
    let stdin = io::stdin();
    let mut last_tag: Option<String> = None;
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
            println!("last tag {:?}", last_tag);
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
        } else {
            break;
        }
        answer.truncate(0);
        println!("{:?}", cards);
        // thread::sleep(time::Duration::from_millis(1000));
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    }

    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}
