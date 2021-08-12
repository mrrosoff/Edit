use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout, BufReader, Lines, Result, Write};
use std::path::Path;

use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::screen::*;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

struct TerminalInformation {
    file_name: String,
    saved: bool,
    row: u32,
    col: u32,
}

fn get_file_name() -> String {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => String::new(),
        2 => String::from(&args[1]),
        _ => {
            print_help();
            String::new()
        }
    }
}

fn print_help() {
    println!("Usage: edit [OPTIONS] [FILE]\n");
    println!("Option\tLong\tMeaning");
}

fn load_file(file_name: String) -> Vec<String> {
    let mut file_lines: Vec<String> = Vec::new();
    if let Ok(lines) = read_lines(Path::new(file_name.as_str())) {
        for line in lines {
            if let Ok(iterator) = line {
                file_lines.push(iterator);
            }
        }
    }
    file_lines
}

fn read_lines(filename: &Path) -> Result<Lines<BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

fn create_editor_ui() -> termion::screen::AlternateScreen<
    termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
> {
    let mut screen = create_screen_overlay();
    write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();
    write!(screen, "Edit").unwrap();
    screen.flush().unwrap();
    screen
}

fn create_screen_overlay() -> termion::screen::AlternateScreen<
    termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
> {
    let raw_terminal = stdout().into_raw_mode().unwrap();
    let with_mouse_support = MouseTerminal::from(raw_terminal);
    let screen = AlternateScreen::from(with_mouse_support);
    screen
}

fn display_file(file: Vec<String>, term: &mut TerminalInformation, screen: &mut dyn Write) {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for [line, i] in file.enumerate() {
        write!(screen, "{}", termion::cursor::Goto(1, term.row as u16)).unwrap();
        let ranges: Vec<(Style, &str)> = h.highlight(file[i as usize].as_str(), &ps);
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        println!("{}", escaped);
        screen.flush().unwrap();
        term.row += 1;
    }
}

fn handle_events(screen: &mut dyn Write) {
    let stdin = stdin();
    let mut cursor_row = 1;
    let mut cursor_col = 1;
    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Ctrl('q')) => break,
            Event::Key(Key::Ctrl('s')) => {
                save_file("Hi.txt");
            }
            Event::Key(Key::Char(c)) => {
                if c as i32 == 10 {
                    cursor_row += 1;
                    cursor_col = 0;
                }
                cursor_col += 1;
                print!("{}", c)
            }
            Event::Key(Key::Left) => cursor_col -= 1,
            Event::Key(Key::Right) => cursor_col += 1,
            Event::Key(Key::Up) => cursor_row -= 1,
            Event::Key(Key::Down) => cursor_row += 1,
            Event::Key(Key::Backspace) => {
                cursor_col -= 1;
            }
            Event::Mouse(me) => match me {
                MouseEvent::Press(_, x, y) => {
                    write!(screen, "{}", termion::cursor::Goto(x, y)).unwrap();
                    screen.flush().unwrap();
                }
                _ => (),
            },
            _ => {}
        }
        write!(screen, "{}", termion::cursor::Goto(cursor_col, cursor_row)).unwrap();
        screen.flush().unwrap();
    }
}

//Figure Out Buffer!
fn save_file(path: &str) -> File {
    let file = match File::create(Path::new(path)) {
        Err(why) => panic!("couldn't create {}: {}", path, why),
        Ok(file) => file,
    };
    file
}

fn main() {
    let term = TerminalInformation {
        file_name: get_file_name(),
        row: 3,
        col: 1,
        saved: false,
    };

    let file;
    if term.file_name != "" {
        file = load_file(term.file_name);
    } else {
        file = Vec::new();
    }

    let mut screen = create_editor_ui();
    display_file(file, &mut term, &mut screen);
    handle_events(&mut screen);
}
