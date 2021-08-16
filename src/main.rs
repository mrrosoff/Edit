use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout, BufReader, Lines, Result, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;

use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::screen::*;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::as_24_bit_terminal_escaped;

struct EditorConfiguration<'a, 'b> {
    syntax: &'b SyntaxReference,
    theme: &'a Theme,
}

struct EditorStatus {
    display_begin_row: u16,
    display_end_row: u16,
    cursor_row: u16,
    cursor_col: u16,
    saved: bool,
}

struct FileInformation {
    file_path: PathBuf,
    file_name: String,
    contents: Vec<String>,
}

struct Editor<'a, 'b> {
    edit_configuration: EditorConfiguration<'a, 'b>,
    edit_status: EditorStatus,
    file_information: FileInformation,
}

impl Editor<'_, '_> {
    fn load_file(&self) {
        let file_lines: Mutex<Vec<String>> = Mutex::new(Vec::new());
        let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
        if let Ok(lines) = self.read_lines(&self.file_information.file_path) {
            let syntax: Mutex<SyntaxReference> = Mutex::new(self.edit_configuration.syntax.clone());
            let theme: Mutex<Theme> = Mutex::new(self.edit_configuration.theme.clone());
            for line in lines {
                let handle = thread::spawn(move || {
                    if let Ok(iterator) = line {
                        let m_syntax = syntax.lock().unwrap();
                        let m_theme = theme.lock().unwrap();
                        let mut h = HighlightLines::new(&m_syntax, &m_theme);
                        let ranges: Vec<(Style, &str)> =
                            h.highlight(iterator.as_str(), &SyntaxSet::load_defaults_newlines());
                        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                        let mut m_file_lines = file_lines.lock().unwrap();
                        m_file_lines.push(escaped);
                    }
                });
                threads.push(handle);
            }
        }
        for thread in threads {
            thread.join().unwrap();
        }
    }

    fn read_lines(&self, filename: &Path) -> Result<Lines<BufReader<File>>> {
        let file = File::open(filename)?;
        Ok(BufReader::new(file).lines())
    }
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

fn display_file(editor: &mut Editor, screen: &mut dyn Write) {
    let mut editor_status = &mut editor.edit_status;
    let file_information = &editor.file_information;

    for line in &file_information.contents
        [editor_status.display_begin_row as usize..editor_status.display_end_row as usize]
    {
        write!(
            screen,
            "{}",
            termion::cursor::Goto(1, editor_status.cursor_row as u16)
        )
        .unwrap();
        println!("{}", line);
        editor_status.cursor_row += 1;
    }
    screen.flush().unwrap();
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

fn save_file(path: &str) -> File {
    let file = match File::create(Path::new(path)) {
        Err(why) => panic!("couldn't create {}: {}", path, why),
        Ok(file) => file,
    };
    file
}

fn main() {
    let file_name = get_file_name();
    //TODO: Fix panic
    let (_, file_extension) = file_name.split_at(file_name.find('.').unwrap() + 1);
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax = syntax_set.find_syntax_by_extension(file_extension).unwrap();
    let theme = &ThemeSet::load_defaults().themes["base16-ocean.dark"];

    let mut editor = Editor {
        edit_configuration: EditorConfiguration {
            syntax: syntax,
            theme: theme,
        },
        edit_status: EditorStatus {
            display_begin_row: 0,
            display_end_row: 3,
            cursor_row: 3,
            cursor_col: 1,
            saved: false,
        },
        file_information: FileInformation {
            file_path: PathBuf::from(file_name.as_str()),
            file_name: file_name,
            contents: Vec::new(),
        },
    };

    editor.load_file();
    let mut screen = create_editor_ui();
    display_file(&mut editor, &mut screen);
    handle_events(&mut screen);
}
