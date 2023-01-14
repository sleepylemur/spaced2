use anyhow::Error;
use cards::Card;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use history::load_history;
use rand::Rng;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{self, Write},
    ops::Deref,
    thread,
    time::{self, Duration, SystemTime, UNIX_EPOCH},
};

mod cards;
mod history;

fn get_possible<'a>(
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

fn random_card<'a>(
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
    load_history("history", &mut cards)?;

    let mut stdout = io::stdout();
    let mut history_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("history")?;
    execute!(stdout, EnterAlternateScreen)?;
    let mut answer = String::new();
    let stdin = io::stdin();
    let mut history_num = 0;
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

            card.update(current_timestamp()?, is_correct, &last_tag, history_num);

            writeln!(
                history_file,
                "{} {} {}",
                card.last_ts,
                card.tag,
                if is_correct { 't' } else { 'f' }
            )?;

            last_tag = Some(card.tag.clone());
        } else {
            break;
        }
        answer.truncate(0);
        history_num += 1;
        println!("{:?}", cards);
        // thread::sleep(time::Duration::from_millis(1000));
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    }

    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}
