use anyhow::Error;
use card_selection::get_possible;
use cards::Card;
use crossterm::{
    cursor, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use history::History;
use std::{collections::HashMap, env, fs, io};

mod card_selection;
mod cards;
mod deck;
mod history;
mod quiz;

fn get_summary(filename: &str) -> Result<String, Error> {
    let mut cards: HashMap<String, Card> = cards::Card::from_file(&format!("cards/{}", filename))?
        .into_iter()
        .map(|card| (card.tag.clone(), card))
        .collect();

    let mut history = History::open(&format!("history/{}", filename))?;
    history.parse(&mut cards)?;
    let (_possible, inactive_count, to_review_count) = get_possible(&mut cards, &None);
    return Ok(format!(
        "{} total: {} reviewing: {} unlearned: {}",
        filename,
        cards.len(),
        to_review_count,
        inactive_count
    ));
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let entries = fs::read_dir("cards")?;
        for entry in entries {
            println!(
                "{}",
                (get_summary(&entry?.file_name().into_string().unwrap())?)
            );
        }
    } else if args[1].len() > 0 {
        let mut stdout = io::stdout();
        let stdin = io::stdin();
        execute!(stdout, EnterAlternateScreen)?;

        quiz::quiz(&args[1], &mut stdout, &stdin)?;

        execute!(stdout, cursor::Show)?;
        execute!(stdout, LeaveAlternateScreen)?;
    }
    Ok(())
}
