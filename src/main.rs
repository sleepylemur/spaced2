use anyhow::Error;
use cards::Card;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    collections::HashMap,
    io::{self},
    time::{SystemTime, UNIX_EPOCH},
};

mod card_selection;
mod cards;
mod history;
mod quiz;

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
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    execute!(stdout, EnterAlternateScreen)?;

    quiz::quiz("cards/ex", &mut stdout, &stdin)?;

    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}
