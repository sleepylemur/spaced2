use cards::Card;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{self, Error, Write},
    ops::Deref,
    thread,
    time::{self, Duration, SystemTime, UNIX_EPOCH},
};

mod cards;

fn get_possible<'a>(cards: &HashMap<String, Card>, last_tag: &Option<String>) -> Vec<String> {
    let mut possible = vec![];
    let now = SystemTime::now();
    for (_, card) in cards {
        if let Some(tag) = last_tag {
            if card.tag.deref() == tag || card.last_followed == *last_tag {
                continue;
            }
        }
        if card.num_correct < 3 {
            possible.push(card.tag.clone());
        } else {
            let hours_passed = now
                .duration_since(UNIX_EPOCH + Duration::from_millis(card.last_ts))
                .unwrap()
                .as_secs()
                / 60
                / 60;
            if u64::from(card.expected_retention_days * 24 - 12) < hours_passed {
                possible.push(card.tag.clone());
            }
        }
    }
    println!("{:?}", possible);
    thread::sleep(time::Duration::from_millis(1000));
    possible
}

fn get_next<'a>(
    cards: &'a mut HashMap<String, Card>,
    last_tag: &Option<String>,
) -> Option<&'a mut Card> {
    thread::sleep(time::Duration::from_millis(1000));
    let possible = get_possible(cards, last_tag);
    if possible.len() == 0 {
        None
    } else {
        let i = rand::thread_rng().gen_range(0..possible.len());
        cards.get_mut(&possible[i])
    }
}

fn main() -> Result<(), Error> {
    let mut cards: HashMap<String, Card> = cards::Card::from_file("cards/ex.txt")?
        .into_iter()
        .map(|card| (card.tag.clone(), card))
        .collect();
    let mut stdout = io::stdout();
    let mut history_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("history.txt")?;
    execute!(stdout, EnterAlternateScreen)?;
    let mut answer = String::new();
    let stdin = io::stdin();
    let mut is_correct;
    let mut history_num = 0;
    let mut last_tag: Option<String> = None;
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    while let Some(card) = get_next(&mut cards, &last_tag) {
        println!("last tag {:?}", last_tag);
        println!("{}", card.question);
        stdin.read_line(&mut answer)?;
        if answer.len() == 0 {
            break;
        }

        if answer.trim() == card.answer {
            println!("correct!");
            is_correct = true;
            card.num_correct += 1;
            if card.num_correct >= 3 {
                if card.expected_retention_days == 0 {
                    card.expected_retention_days = 1;
                } else {
                    card.expected_retention_days *= 2;
                }
            }
        } else {
            println!("nope");
            is_correct = false;
            card.num_correct = 0;
            card.expected_retention_days /= 2;
        }

        card.last_num = history_num;
        card.last_ts = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
        .unwrap();
        card.last_followed = last_tag;

        writeln!(
            history_file,
            "{} {} {}",
            card.last_ts,
            card.tag,
            if is_correct { 't' } else { 'f' }
        )?;

        last_tag = Some(card.tag.clone());
        answer.truncate(0);
        history_num += 1;
        println!("{:?}", cards);
        thread::sleep(time::Duration::from_millis(1000));
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    }

    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}
