use std::env;
use std::fs;
use std::io;
use std::io::{stdin, stdout, Write};
use std::path::{Path, PathBuf};

use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::screen::*;
use termion::terminal_size;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::as_24_bit_terminal_escaped;

const EDITOR_NAME_OFFSET: usize = 2;

struct EditorConfiguration<'a, 'b> {
    syntax: &'b SyntaxReference,
    theme: &'a Theme,
}

struct EditorStatus {
    width: usize,
    height: usize,
    display_begin_row: usize,
    display_end_row: usize,
    cursor_row: usize,
    cursor_col: usize,
    saved: bool,
}

struct FileInformation {
    file_path: PathBuf,
    file_name: String,
    contents: Vec<String>,
}

struct Editor<'a, 'b> {
    editor_configuration: EditorConfiguration<'a, 'b>,
    editor_status: EditorStatus,
    file_information: FileInformation,
}

impl Editor<'_, '_> {
    fn load_file(&mut self) {
        let file = fs::read_to_string(&self.file_information.file_path).unwrap();
        let mut highlighter = HighlightLines::new(
            &self.editor_configuration.syntax,
            &self.editor_configuration.theme,
        );
        let ranges: Vec<(Style, &str)> =
            highlighter.highlight(file.as_str(), &SyntaxSet::load_defaults_newlines());
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        let split_string = escaped.lines();
        let highlighted_lines: Vec<String> = split_string.map(|s| s.to_string()).collect();
        self.file_information.contents = highlighted_lines.clone();
        if self.editor_status.display_end_row > self.file_information.contents.len() {
            self.editor_status.display_end_row =
                self.file_information.contents.len() + EDITOR_NAME_OFFSET;
        }
    }
}

fn get_file_name() -> Result<String, io::Error> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => Ok(String::new()),
        2 => Ok(String::from(&args[1])),
        _ => Err(io::Error::from(io::ErrorKind::InvalidInput))
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

fn repaint_file(editor: &mut Editor, screen: &mut dyn Write) {
    let editor_status = &mut editor.editor_status;
    let file_information = &editor.file_information;
    write!(screen, "{}", termion::clear::All).unwrap();
    let mut write_row = EDITOR_NAME_OFFSET as u16;
    for line in &file_information.contents
        [editor_status.display_begin_row..editor_status.display_end_row - EDITOR_NAME_OFFSET]
    {
        write!(screen, "{}{}", termion::cursor::Goto(1, write_row), line).unwrap(); //need to hide the cursor termion::cursor::HideCursor
        write_row += 1;
    }
    write!(
        screen,
        "{}",
        termion::cursor::Goto(
            editor_status.cursor_col as u16,
            editor_status.cursor_row as u16
        )
    )
    .unwrap();
    screen.flush().unwrap();
}

fn repaint_movement(editor: &mut Editor, screen: &mut dyn Write) {
    let editor_status = &mut editor.editor_status;

    write!(
        screen,
        "{}",
        termion::cursor::Goto(
            editor_status.cursor_col as u16,
            editor_status.cursor_row as u16
        )
    )
    .unwrap();
    screen.flush().unwrap();
}

fn handle_events(editor: &mut Editor, screen: &mut dyn Write) {
    let stdin = stdin();
    for c in stdin.events() {
        let input = c.unwrap();
        if handle_editing(editor, screen, &input)
            || handle_key_movements(editor, screen, &input)
            || handle_hot_keys(&input)
            || handle_special_movements(screen, &input)
        {
            continue;
        } else if input == Event::Key(Key::Ctrl('q')) {
            break;
        }
    }
}

fn handle_editing(
    editor: &mut Editor,
    screen: &mut dyn Write,
    input: &termion::event::Event,
) -> bool {
    match input {
        Event::Key(Key::Char(c)) => {
            if *c as i32 == 10 {
                editor.editor_status.cursor_row += 1;
                editor.editor_status.cursor_col = 0;
            }
            editor.editor_status.cursor_col += 1;
            print!("{}", c);
            screen.flush().unwrap();
        }
        _ => {
            return false;
        }
    }
    return true;
}

fn handle_key_movements(
    editor: &mut Editor,
    screen: &mut dyn Write,
    input: &termion::event::Event,
) -> bool {
    match input {
        Event::Key(Key::Left) => {
            if editor.editor_status.cursor_col != 0 {
                editor.editor_status.cursor_col -= 1;
                repaint_movement(editor, screen);
            }
        }
        Event::Key(Key::Right) => {
            if editor.editor_status.cursor_col != editor.editor_status.width - 1 {
                editor.editor_status.cursor_col += 1;
                repaint_movement(editor, screen);
            }
        }
        Event::Key(Key::Up) => {
            if editor.editor_status.cursor_row != 0 {
                if editor.editor_status.cursor_row == editor.editor_status.display_begin_row {
                    editor.editor_status.display_begin_row -= 1;
                    editor.editor_status.display_end_row -= 1;
                    repaint_file(editor, screen);
                }
                editor.editor_status.cursor_row -= 1;
                repaint_movement(editor, screen);
            }
        }
        Event::Key(Key::Down) => {
            if editor.editor_status.cursor_row
                != editor.file_information.contents.len() + EDITOR_NAME_OFFSET
            {
                if editor.editor_status.cursor_row == editor.editor_status.display_end_row {
                    editor.editor_status.display_begin_row += 1;
                    editor.editor_status.display_end_row += 1;
                    repaint_file(editor, screen);
                }
                editor.editor_status.cursor_row += 1;
                repaint_movement(editor, screen);
            }
        }
        Event::Key(Key::Backspace) => {
            if editor.editor_status.cursor_col != 0 {
                editor.editor_status.cursor_col -= 1;
                repaint_movement(editor, screen);
            }
        }
        _ => {
            return false;
        }
    }
    return true;
}

fn handle_hot_keys(input: &termion::event::Event) -> bool {
    match input {
        Event::Key(Key::Ctrl('s')) => {
            save_file("Hi.txt");
        }
        _ => {
            return false;
        }
    }
    return true;
}

fn handle_special_movements(screen: &mut dyn Write, input: &termion::event::Event) -> bool {
    match input {
        Event::Mouse(me) => match me {
            MouseEvent::Press(_, x, y) => {
                write!(screen, "{}", termion::cursor::Goto(*x, *y)).unwrap();
                screen.flush().unwrap();
            }
            _ => {}
        },
        _ => {
            return false;
        }
    }
    return true;
}

fn save_file(path: &str) -> fs::File {
    let file = match fs::File::create(Path::new(path)) {
        Err(why) => panic!("couldn't create {}: {}", path, why),
        Ok(file) => file,
    };
    file
}

fn main() {
    let file_name = match get_file_name() {
        Ok(file_name) => file_name,
        Err(_) => {
            print_help();
            return;
        }
    };

    let file_extension = match file_name.find('.') {
        Some(index) => file_name.split_at(index + 1).1,
        None => "txt",
    };

    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax = syntax_set.find_syntax_by_extension(file_extension).unwrap();
    let theme = &ThemeSet::load_defaults().themes["base16-ocean.dark"];
    let terminal_size = terminal_size().unwrap();

    let mut editor = Editor {
        editor_configuration: EditorConfiguration {
            syntax: syntax,
            theme: theme,
        },
        editor_status: EditorStatus {
            width: terminal_size.0 as usize,
            height: terminal_size.1 as usize,
            display_begin_row: 0,
            display_end_row: terminal_size.1 as usize,
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

    if editor.file_information.file_name != "" {
        editor.load_file();
    }

    let mut screen = create_editor_ui();
    repaint_file(&mut editor, &mut screen);
    handle_events(&mut editor, &mut screen);
}
