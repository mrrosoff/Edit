use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};
use termion::raw::IntoRawMode;
use termion::screen::*;

enum TerminalMode {
    NewFile,
    EditFile(String),
    None,
}

fn write_alt_screen_msg<W: Write>(screen: &mut W) {
    write!(screen, "{}{}Welcome to the alternate screen.{}Press '1' to switch to the main screen or '2' to switch to the alternate screen.{}Press 'q' to exit (and switch back to the main screen).",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Goto(1, 3),
           termion::cursor::Goto(1, 4)).unwrap();
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
    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(screen, "{}", termion::cursor::Hide).unwrap();
    print!("{}", file);
    write_alt_screen_msg(&mut screen);
    screen.flush().unwrap();
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
