use std::io::{Stdin, Stdout};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Error};
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};

use crate::deck::Deck;

fn clear_and_print_title(
    stdout: &mut Stdout,
    total: usize,
    inactive: usize,
    to_review: usize,
) -> Result<(), Error> {
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    println!(
        "total: {} reviewing: {} unlearned: {}\n",
        total, to_review, inactive
    );
    Ok(())
}

fn add_cards(stdin: &Stdin, deck: &mut Deck) -> Result<QuizState, Error> {
    let mut answer = String::new();
    if deck.inactive > 0 {
        println!("-- a to add. Anything else to exit --");
        stdin.read_line(&mut answer)?;
        if answer.trim() == "a" {
            deck.activate_cards(5);
            Ok(QuizState::First)
        } else {
            Ok(QuizState::Quit)
        }
    } else {
        println!("-- enter anything to exit --");
        stdin.read_line(&mut answer)?;
        Ok(QuizState::Quit)
    }
}

fn ask_question(stdin: &Stdin, deck: &mut Deck, last_correct: bool) -> Result<QuizState, Error> {
    match deck.current_card() {
        None => Err(anyhow!("empty card passed to ask_question")),
        Some(card) => {
            if last_correct {
                println!("correct");
            }
            println!("{}", card.question);
            let mut answer = String::new();
            stdin.read_line(&mut answer)?;
            if answer.len() == 0 {
                Ok(QuizState::Quit)
            } else if answer.trim() == card.answer {
                deck.answer(true)?;
                Ok(QuizState::Correct)
            } else {
                deck.answer(false)?;
                Ok(QuizState::ReviewWrongAnswer)
            }
        }
    }
}

fn review_wrong_answer(stdin: &Stdin, deck: &Deck) -> Result<QuizState, Error> {
    match deck.current_card() {
        None => Err(anyhow!("empty card passed to review_mistake")),
        Some(card) => {
            println!(
                "wrong\n{}\n{}\n-- enter anything to continue --",
                card.question, card.answer
            );
            let mut answer = String::new();
            stdin.read_line(&mut answer)?;
            if answer.len() == 0 {
                Ok(QuizState::Quit)
            } else {
                Ok(QuizState::RetryWrongAnswer)
            }
        }
    }
}

#[derive(PartialEq)]
enum QuizState {
    Quit,
    Empty,
    First,
    Correct,
    ReviewWrongAnswer,
    RetryWrongAnswer,
}

pub fn quiz(
    base_path: &PathBuf,
    filename: &str,
    stdout: &mut Stdout,
    stdin: &Stdin,
) -> Result<(), Error> {
    println!(
        "quiz filename {} base_path {}",
        filename,
        base_path.to_str().context("failed to convert base_path")?
    );
    let mut deck = Deck::load(&base_path, &filename)?;

    let mut state = QuizState::First;

    while QuizState::Quit != state {
        deck.update_possible();
        if deck.reviewing == 0 {
            state = QuizState::Empty
        }
        clear_and_print_title(stdout, deck.len(), deck.inactive, deck.reviewing)?;

        state = match state {
            QuizState::Quit => break,
            QuizState::Empty => add_cards(stdin, &mut deck)?,
            QuizState::First => {
                deck.next();
                ask_question(stdin, &mut deck, false)?
            }
            QuizState::Correct => {
                deck.next();
                ask_question(stdin, &mut deck, true)?
            }
            QuizState::ReviewWrongAnswer => review_wrong_answer(stdin, &deck)?,
            QuizState::RetryWrongAnswer => ask_question(stdin, &mut deck, false)?,
        }
    }
    Ok(())
}
