use std::env;
use std::fs;
use std::io::{Write, stdout, stdin};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;

fn create_file_buffer() -> String {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => String::from("newFile.txt"),
        2 => {
            let filename = &args[1];
            let data = fs::read_to_string(filename).expect("Unable to read file");
            filename.clone()
        }
        _ => String::from("Error!"),
    }
}

fn create_alternate_screen(file: String) -> termion::screen::AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>> {
    let stdin = stdin();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();
    screen.flush().unwrap();
    screen
}

fn iterate_key_strokes(screen: &mut Write) {
    let stdin = stdin();
    let mut cursor_row = 1;
    let mut cursor_col = 1;
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl('q') => break,
            Key::Char(c) => {
                if c as i32 == 10 {
                    cursor_row += 1;
                    cursor_col = 0;
                }
                cursor_col += 1;
                print!("{}", c)
            }
            Key::Left => cursor_col -= 1,
            Key::Right => cursor_col += 1,
            Key::Up => cursor_row -= 1,
            Key::Down => cursor_row += 1,
            Key::Backspace => {
                cursor_col -= 1;
            }
            _ => {}
        }
        write!(screen, "{}", termion::cursor::Goto(cursor_col, cursor_row)).unwrap();
        screen.flush().unwrap();
    }
}

fn clean_up(screen: &mut Write) {
    write!(screen, "{}", termion::cursor::Show).unwrap();
}

fn main() {
    let file_buffer = create_file_buffer();
    let mut screen = create_alternate_screen(file_buffer);
    iterate_key_strokes(&mut screen);
    clean_up(&mut screen);
}
