use std::io::{stdin, stdout, Write};
use tokio::sync::mpsc;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use rand::Rng;
use std::time::Duration;
use std::fs;
use termion::cursor::Hide;
use termion::{cursor, style};

fn print_bordered_file(x: u16, y: u16, filename: &str, border_char: Option<char>) {
    let content = fs::read_to_string(filename)
        .expect("Failed to read file");
    let border = border_char.unwrap_or('#');

    // Calculate dimensions
    let max_length = content.lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let border_line = std::iter::repeat(border).take(max_length + 4).collect::<String>();

    // Prepare bordered content
    let mut bordered_content = vec![border_line.clone()];
    for line in content.lines() {
        bordered_content.push(format!(
            "{} {: <width$} {}",
            border,
            line,
            border,
            width = max_length
        ));
    }
    bordered_content.push(border_line);

    // Print with positioning
    let mut stdout = stdout();
    for (i, line) in bordered_content.iter().enumerate() {
        write!(stdout, "{}{}", cursor::Goto(x, y + i as u16), line)
            .expect("Failed to write");
    }
    stdout.flush().unwrap();
}

async fn key_interrupt_listener(tx: mpsc::Sender<Key>) {
    let stdin = stdin();
    let _stdout = stdout().into_raw_mode().unwrap(); // Keep raw mode alive

    for key in stdin.keys() {
        if let Ok(key) = key {
            if tx.send(key).await.is_err() {
                break; // Stop if receiver is dropped
            }
        }
    }
}

fn roll() {
    let reels = 11;
    let mut rng = rand::thread_rng();
    let (width, height) = termion::terminal_size().unwrap();

    for n in (1..120).step_by(40) {
        let rand_nr = rng.gen_range(1..=reels);
        print_bordered_file((width / 2) - 60 + n, (height / 2) - 6, "ascii_art/0.txt", Some('#'));
        std::thread::sleep(Duration::new(0, 200_000_000));
        print_bordered_file((width / 2) - 60 + n, (height / 2) - 6, &*format!("ascii_art/{}.txt", rand_nr), Some('#'));
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    // Spawn key listener task
    tokio::spawn(key_interrupt_listener(tx));

    // Clear screen and hide cursor
    print!("{}[2J", 27 as char);
    write!(stdout(), "{}", Hide).unwrap();

    let mut running = true;

    while running {
        tokio::select! {
            // Handle key events
            Some(key) = rx.recv() => {
                match key {
                    Key::Char(' ') => {
                        running = false;
                        roll();
                    }
                    Key::Ctrl('c') => {
                        running = false;
                    }
                    _ => running = true,
                }
            }

            // Main animation loop
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                roll();
            }
        }
    }
}
