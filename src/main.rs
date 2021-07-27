use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout, Write};
use std::path::Path;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;

fn write_alt_screen_msg<W: Write>(screen: &mut W) {
    write!(screen, "{}", termion::clear::All).unwrap();
    write!(screen, "{}", termion::cursor::Goto(1, 1)).unwrap();
    write!(screen, "Welcome To The Alternate Screen.").unwrap();
}

//Figure Out Buffer!
fn save_file(path: &str) -> File {
    let file = match File::create(Path::new(path)) {
        Err(why) => panic!("couldn't create {}: {}", path, why),
        Ok(file) => file,
    };
    file
}

// fn determine_terminal_mode() -> File {
//     let args: Vec<String> = env::args().collect();
//     match args.len() {
//         2 => {
//             let path = Path::new(&args[1]);
//             let file = match File::create(&path) {
//                 Err(why) => panic!("couldn't create {}: {}" , &args[1], why),
//                 Ok(file) => file,
//             };
//             file
//         }
//         _ => panic!("usage: {} <file>", &args[0]);
//     }
// }

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
    // let terminal_mode = determine_terminal_mode();
    print!("Done!")
    // match terminal_mode {
    //     TerminalMode::NewFile => {
    //         create_alternate_screen(String::new());
    //     }
    //     TerminalMode::EditFile(filename) => {
    //         let data = fs::read_to_string(filename).expect("Unable to read file");
    //         create_alternate_screen(data);
    //     }
    //     TerminalMode::None => {
    //         return;
    //     }
    // }
}
