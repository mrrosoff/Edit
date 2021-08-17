use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout, BufReader, Lines, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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
    display_begin_row: usize,
    display_end_row: usize,
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
    fn load_file(&mut self) {
        let lines = self.read_lines(&self.file_information.file_path);
        let syntax = Arc::new(Mutex::new(self.edit_configuration.syntax.clone()));
        let theme = Arc::new(Mutex::new(self.edit_configuration.theme.clone()));

        let file_lines = Arc::new(Mutex::new(Vec::new()));
        let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();

        for line in lines {
            let file_lines = Arc::clone(&file_lines);
            let syntax = Arc::clone(&syntax);
            let theme = Arc::clone(&theme);
            let handle = thread::spawn(move || {
                if let Ok(iterator) = line {
                    let syntax_ref = syntax.lock().unwrap();
                    let theme_ref = theme.lock().unwrap();
                    let mut h = HighlightLines::new(&*syntax_ref, &*theme_ref);
                    let ranges: Vec<(Style, &str)> =
                        h.highlight(iterator.as_str(), &SyntaxSet::load_defaults_newlines());
                    let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                    let mut file_lines_ref = file_lines.lock().unwrap();
                    file_lines_ref.push(escaped);
                }
            });
            threads.push(handle);
        }
        for thread in threads {
            thread.join().unwrap();
        }
        self.file_information.contents = (*file_lines.lock().unwrap()).clone();
    }

    fn read_lines(&self, filename: &Path) -> Lines<BufReader<File>> {
        let file = File::open(filename).unwrap();
        BufReader::new(file).lines()
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
    let editor_status = &mut editor.edit_status;
    let file_information = &editor.file_information;

    let mut write_row = 0;
    for line in
        &file_information.contents[editor_status.display_begin_row..editor_status.display_end_row]
    {
        write!(screen, "{}{}", termion::cursor::Goto(1, write_row), line).unwrap();
        write_row += 1;
    }
    screen.flush().unwrap();
}

fn handle_events(editor: &mut Editor, screen: &mut dyn Write) {
    let stdin = stdin();
    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Ctrl('q')) => break,
            Event::Key(Key::Ctrl('s')) => {
                save_file("Hi.txt");
            }
            Event::Key(Key::Char(c)) => {
                if c as i32 == 10 {
                    editor.edit_status.cursor_row += 1;
                    editor.edit_status.cursor_col = 0;
                }
                editor.edit_status.cursor_col += 1;
                print!("{}", c)
            }
            Event::Key(Key::Left) => editor.edit_status.cursor_col -= 1,
            Event::Key(Key::Right) => editor.edit_status.cursor_col += 1,
            Event::Key(Key::Up) => editor.edit_status.cursor_row -= 1,
            Event::Key(Key::Down) => editor.edit_status.cursor_row += 1,
            Event::Key(Key::Backspace) => {
                editor.edit_status.cursor_col -= 1;
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
        write!(
            screen,
            "{}",
            termion::cursor::Goto(editor.edit_status.cursor_col, editor.edit_status.cursor_row)
        )
        .unwrap();
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
            display_end_row: 10,
            cursor_row: 2,
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
    handle_events(&mut editor, &mut screen);
}
