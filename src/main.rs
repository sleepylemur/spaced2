use anyhow::Error;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

mod card_selection;
mod cards;
mod history;
mod quiz;

fn main() -> Result<(), Error> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    execute!(stdout, EnterAlternateScreen)?;

    quiz::quiz("ex", &mut stdout, &stdin)?;

    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}
