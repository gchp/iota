use std::collections::HashMap;
use std::cmp;

use unicode_width::UnicodeWidthChar;
use rustbox::{Style, Color, RustBox};

use command::{Command, BuilderEvent};
use keyboard::Key;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OverlayType {
    CommandPrompt,
}

pub trait Overlay {
    fn draw(&self, rb: &mut RustBox);
    fn draw_cursor(&mut self, rb: &mut RustBox);
    fn handle_key_event(&mut self, key: Key) -> BuilderEvent;
}

pub struct CommandPrompt {
    cursor_x: usize,
    data: String,
    prefix: String,
    commands: HashMap<String, Command>,
    selected_index: usize,
}

impl CommandPrompt {
    pub fn new() -> CommandPrompt {
        let mut commands = HashMap::new();

        commands.insert("quit".into(), Command::exit_editor());
        commands.insert("write".into(), Command::save_buffer());

        CommandPrompt {
            cursor_x: 1,
            data: String::new(),
            prefix: String::from(":"),
            commands: commands,
            selected_index: 0,
        }
    }
}

impl CommandPrompt {
    fn get_filtered_command_names(&self) -> Vec<&String> {
        let mut keys: Vec<&String> = self.commands
            .keys()
            .filter(|ref item| (&item).starts_with(&self.data) )
            .collect();
        keys.sort();
        keys.reverse();

        keys
    }
}


impl Overlay for CommandPrompt {
    fn draw(&self, rb: &mut RustBox) {
        let height = rb.height() - 1;
        let offset = self.prefix.len();

        let keys = self.get_filtered_command_names();

        // find the longest command in the resulting list
        let mut max = 20;
        for k in &keys {
            max = cmp::max(max, k.len());
        }

        // draw the command completion list
        let mut index = 1;
        for key in &keys {
            rb.print_char(0, height - index, Style::empty(), Color::White, Color::Black, '│');
            rb.print_char(max + 1, height - index, Style::empty(), Color::White, Color::Black, '│');

            let (fg, bg) = if index == self.selected_index {
                (Color::White, Color::Red)
            } else {
                (Color::White, Color::Black)
            };

            let mut chars = key.chars();
            for x in 0..max {
                if let Some(ch) = chars.next() {
                    rb.print_char(x + 1, height - index, Style::empty(), fg, bg, ch);
                } else {
                    rb.print_char(x + 1, height - index, Style::empty(), fg, bg, ' ');
                }
            }

            index += 1;
        }

        rb.print_char(0, height - index, Style::empty(), Color::White, Color::Black, '╭');
        for x in 1..max + 1 {
            rb.print_char(x, height - keys.len() - 1, Style::empty(), Color::White, Color::Black, '─');
        }
        rb.print_char(max + 1, height - index, Style::empty(), Color::White, Color::Black, '╮');

        // draw the given prefix
        for (index, ch) in self.prefix.chars().enumerate() {
            rb.print_char(index, height, Style::empty(), Color::White, Color::Black, ch);
        }

        // draw the overlay data
        for (index, ch) in self.data.chars().enumerate() {
            rb.print_char(index + offset, height, Style::empty(), Color::White, Color::Black, ch);
        }
    }

    fn draw_cursor(&mut self, rb: &mut RustBox) {
        // Prompt is always on the bottom, so we can use the
        // height given by the frontend here
        let height = rb.height() - 1;
        rb.set_cursor(self.cursor_x as isize, height as isize)
    }

    fn handle_key_event(&mut self, key: Key) -> BuilderEvent {
        match key {
            Key::Esc => return BuilderEvent::Complete(Command::noop()),
            Key::Backspace => {
                if let Some(c) = self.data.pop() {
                    if let Some(width) = UnicodeWidthChar::width(c) {
                        self.cursor_x -= width;
                    }
                }
            }
            Key::Enter => {
                match self.commands.get(&self.data) {
                    Some(command) => {
                        return BuilderEvent::Complete(*command);
                    }
                    None => {
                        return BuilderEvent::Incomplete;
                    }
                }
            }
            Key::Up => {
                let max = self.get_filtered_command_names().len();
                if self.selected_index < max {
                    self.selected_index += 1;
                }
            }
            Key::Down => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            Key::Tab => {
                if self.selected_index > 0 {
                    let command = {
                        let keys = self.get_filtered_command_names();
                        keys[self.selected_index - 1].clone()
                    };
                    self.data = command;
                    self.cursor_x = self.data.len() + 1;
                }
            }
            Key::Char(c) => {
                if let Some(width) = UnicodeWidthChar::width(c) {
                    self.data.push(c);
                    self.cursor_x += width;
                }
            }
            _ => {}
        }
        return BuilderEvent::Incomplete;
    }
}
