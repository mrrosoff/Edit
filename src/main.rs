use std::env;
use std::fs;
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
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

fn load_file() -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    let mut file_lines: Vec<String> = Vec::new();
    match args.len() {
        2 => {
            if let Ok(lines) = read_lines(Path::new(&args[1])) {
                for line in lines {
                    if let Ok(ip) = line {
                        file_lines.push(ip);
                    }
                }
            }
        }
        _ => {
            println!("WHOOPS")
        }
    }
    file_lines
}

fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

fn create_editor_ui() -> termion::screen::AlternateScreen<
    termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
> {
    let mut screen = create_screen_overlay();
    write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();
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

fn display_file(file: Vec<String>, screen: &mut Write) {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in file {
        // LinesWithEndings enables use of newlines mode
        let ranges: Vec<(Style, &str)> = h.highlight(line.as_str(), &ps);
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        println!("{}", escaped);
    }
}

fn handle_events(screen: &mut Write) {
    let stdin = stdin();
    let mut cursor_row = 1;
    let mut cursor_col = 1;
    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Ctrl('q')) => break,
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
                }
                _ => (),
            },
            _ => {}
        }
        write!(screen, "{}", termion::cursor::Goto(cursor_col, cursor_row)).unwrap();
        screen.flush().unwrap();
    }
}

fn clean_up(screen: &mut Write) {
    save_file("Hi");
    write!(screen, "{}", termion::cursor::Show).unwrap();
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
    let file = load_file();
    let mut screen = create_editor_ui();
    display_file(file, &mut screen);
    handle_events(&mut screen);
    clean_up(&mut screen);
}
