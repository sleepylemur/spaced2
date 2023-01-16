use std::{
    collections::HashMap,
    io::{Stdin, Stdout},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Error};
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};

use crate::{
    card_selection::{get_possible, random_card, random_tag},
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

fn add_cards(
    stdin: &Stdin,
    cards: &mut HashMap<String, Card>,
    has_unlearned: bool,
) -> Result<QuizState, Error> {
    let mut answer = String::new();
    if has_unlearned {
        println!("-- a to add. Anything else to exit --");
        stdin.read_line(&mut answer)?;
        if answer.trim() == "a" {
            activate_cards(cards, 5);
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

fn ask_question(
    stdin: &Stdin,
    cards: &mut HashMap<String, Card>,
    current_tag: &Option<String>,
    last_correct: bool,
    last_tag: &Option<String>,
    history: &mut History,
) -> Result<QuizState, Error> {
    match cards.get_mut(current_tag.as_ref().unwrap()) {
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
                card.update(current_timestamp()?, true, &last_tag, history.num);
                history.persist_update(card, true)?;
                Ok(QuizState::Correct)
            } else {
                card.update(current_timestamp()?, false, &last_tag, history.num);
                history.persist_update(card, false)?;
                Ok(QuizState::ReviewWrongAnswer)
            }
        }
    }
}

fn review_wrong_answer(
    stdin: &Stdin,
    cards: &HashMap<String, Card>,
    current_tag: &Option<String>,
) -> Result<QuizState, Error> {
    match cards.get(current_tag.as_ref().unwrap()) {
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

pub fn quiz(filename: &str, stdout: &mut Stdout, stdin: &Stdin) -> Result<(), Error> {
    let mut cards: HashMap<String, Card> = cards::Card::from_file(&format!("cards/{}", filename))?
        .into_iter()
        .map(|card| (card.tag.clone(), card))
        .collect();

    let mut history = History::open(&format!("history/{}", filename))?;
    history.parse(&mut cards)?;

    let mut state = QuizState::First;
    let mut last_tag: Option<String> = None;
    let mut current_tag = None;
    let num_cards = cards.len();

    while QuizState::Quit != state {
        let (possible, inactive_count, to_review_count) = get_possible(&mut cards, &last_tag);
        if to_review_count == 0 {
            state = QuizState::Empty
        }

        clear_and_print_title(stdout, num_cards, inactive_count, to_review_count)?;

        state = match state {
            QuizState::Quit => break,
            QuizState::Empty => add_cards(stdin, &mut cards, inactive_count > 0)?,
            QuizState::First => {
                current_tag = random_tag(&possible);
                ask_question(
                    stdin,
                    &mut cards,
                    &current_tag,
                    false,
                    &last_tag,
                    &mut history,
                )?
            }
            QuizState::Correct => {
                last_tag = current_tag;
                current_tag = random_tag(&possible);
                ask_question(
                    stdin,
                    &mut cards,
                    &current_tag,
                    true,
                    &last_tag,
                    &mut history,
                )?
            }
            QuizState::ReviewWrongAnswer => review_wrong_answer(stdin, &cards, &current_tag)?,
            QuizState::RetryWrongAnswer => ask_question(
                stdin,
                &mut cards,
                &current_tag,
                false,
                &last_tag,
                &mut history,
            )?,
        }
    }
    Ok(())
}
