use anyhow::{Context, Error};
use card_selection::get_possible;
use cards::Card;
use crossterm::{
    cursor, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use history::History;
use std::{collections::HashMap, env, fs, io};
use std::{ffi::OsStr, path::PathBuf};

mod card_selection;
mod cards;
mod deck;
mod history;
mod quiz;

fn get_base_path() -> std::io::Result<PathBuf> {
    // Navigate to root project dir from target/debug/spaced2
    let mut base_path = std::env::current_exe()?;
    base_path.pop();
    base_path.pop();
    base_path.pop();
    Ok(base_path)
}

fn get_summary(base_path: &PathBuf, filename: &OsStr) -> Result<String, Error> {
    let mut card_path = base_path.clone();
    card_path.push("cards");
    card_path.push(filename);

    let mut cards: HashMap<String, Card> = cards::Card::from_file(&card_path)?
        .into_iter()
        .map(|card| (card.tag.clone(), card))
        .collect();

    let mut history_path = base_path.clone();
    history_path.push("history");
    history_path.push(filename);

    let mut history = History::open(&history_path)?;
    history.parse(&mut cards)?;
    let (_possible, inactive_count, to_review_count) = get_possible(&mut cards, &None);
    return Ok(format!(
        "{} total: {} reviewing: {} unlearned: {}",
        filename.to_str().context("trouble parsing filename")?,
        cards.len(),
        to_review_count,
        inactive_count
    ));
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let base_path = get_base_path()?;
    if args.len() == 1 {
        let mut cards_path = base_path.clone();
        cards_path.push("cards");
        let entries = fs::read_dir(cards_path)?;
        for entry in entries {
            println!("{}", get_summary(&base_path, &entry?.file_name())?);
        }
    } else if args[1].len() > 0 {
        let mut stdout = io::stdout();
        let stdin = io::stdin();
        execute!(stdout, EnterAlternateScreen)?;
        quiz::quiz(&base_path, &args[1], &mut stdout, &stdin)?;

        execute!(stdout, cursor::Show)?;
        execute!(stdout, LeaveAlternateScreen)?;
    }
    Ok(())
}
