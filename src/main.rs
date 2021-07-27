use std::env;
use std::fs;
use std::io::{Write, stdout, stdin};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;

enum TerminalMode {
    NewFile,
    EditFile(String),
    None,
}

fn write_alt_screen_msg<W: Write>(screen: &mut W) {
    write!(screen, "{}", termion::clear::All).unwrap();
    write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();
    write!(screen, "Welcome To The Alternate Screen.").unwrap();
}

fn determine_terminal_mode() -> TerminalMode {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => TerminalMode::NewFile,
        2 => {
            let filename = &args[1];
            TerminalMode::EditFile(filename.clone())
        }
        _ => TerminalMode::None,
    }
}

fn create_alternate_screen(file: String) {
    let stdin = stdin();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(screen, "{}", termion::cursor::Hide).unwrap();
    write_alt_screen_msg(&mut screen);
    screen.flush().unwrap();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char(c) => println!("{}", c),
            Key::Alt(c) => println!("^{}", c),
            Key::Ctrl(c) => println!("*{}", c),
            Key::Esc => println!("ESC"),
            Key::Left => println!("←"),
            Key::Right => println!("→"),
            Key::Up => println!("↑"),
            Key::Down => println!("↓"),
            Key::Backspace => println!("×"),
            _ => {}
        }
        screen.flush().unwrap();
    }
    write!(screen, "{}", termion::cursor::Show).unwrap();   
}

fn main() {
    let terminal_mode = determine_terminal_mode();
    match terminal_mode {
        TerminalMode::NewFile => {
            create_alternate_screen(String::new());
        }
        TerminalMode::EditFile(filename) => {
            let data = fs::read_to_string(filename).expect("Unable to read file");
            create_alternate_screen(data);
        }
        TerminalMode::None => {
            return;
        }
    }
}
