use crate::{
    TextSlice, Style,
    event::Event,
    os::{self, Driver as _},
    scroll_buffer::{self, ScrollBuffer},
};
use std::io;

use crate::os::current::Driver as Driver;

/// A terminal.
pub struct Terminal {
    /// The settings.
    settings: Settings,
    /// The operating-system specific driver.
    os_driver: Driver,
    /// The backing text buffer.
    scroll_buffer: ScrollBuffer,
}

/// Terminal settings.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Settings {
    /// The shell to execute.
    pub shell: String,
    /// How many lines to remember in the scrollback.
    pub lines_to_remember: usize,
    /// The maximum number of lines to display at once.
    pub line_count: usize,
    /// The maximum number of columns to display at once.
    pub column_count: usize,
    /// The number of spaces used to render tab characters.
    pub tab_width: usize,
}

/// A terminal action.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// Writes text into the terminal.
    WriteText(String),
    /// Deletes the previous character.
    Backspace,
    /// The ESC key.
    Escape,
    /// Moves the cursor left.
    CursorLeft,
    /// Moves the cursor right.
    CursorRight,
    /// Moves the cursor down.
    CursorDown,
    /// Moves the cursor up.
    CursorUp,
    /// Sends a control code to the pseudo terminal.
    ControlCode(char),
}

impl Terminal {
    /// Creates a new terminal.
    pub fn new(settings: Settings) -> Result<Self, io::Error> {
        let os_driver = Driver::new(&settings)?;

        Ok(Terminal {
            os_driver,
            scroll_buffer: ScrollBuffer::new(scroll_buffer::Settings {
                lines_to_remember: settings.lines_to_remember,
                max_lines: settings.line_count,
                max_columns: settings.column_count,
                tab_width: settings.tab_width,
            }),
            settings,
        })
    }

    /// Writes text to the terminal.
    pub fn write_text(&mut self, s: &str) {
        self.scroll_buffer.put_str(s);
        self.os_driver.write_text(s);
    }

    /// Backspaces the last character.
    pub fn backspace(&mut self) {
        self.scroll_buffer.backspace();
        self.os_driver.backspace();
    }

    /// Sends the ESC character code.
    pub fn escape(&mut self) {
        self.os_driver.escape();
    }

    /// Moves the cursor left.
    pub fn cursor_left(&mut self) {
        self.os_driver.cursor_left();
    }

    /// Moves the cursor right.
    pub fn cursor_right(&mut self) {
        self.os_driver.cursor_right();
    }

    /// Moves the cursor up.
    pub fn cursor_up(&mut self) {
        self.os_driver.cursor_up();
    }

    /// Moves the cursor down.
    pub fn cursor_down(&mut self) {
        self.os_driver.cursor_down();
    }

    /// Sends a control code to the running process.
    pub fn control_code(&mut self, c: char) {
        self.os_driver.control_code(c);
    }

    /// Sends an interrupt signal to the running program.
    pub fn signal_interrupt(&mut self) {
        self.control_code('c');
    }

    /// Sends raw data to the underlying terminal.
    pub fn send_raw<S>(&mut self, s: S) where S: ToString {
        self.os_driver.send_raw(s);
    }

    /// Updates the terminal.
    pub fn update(&mut self) -> Vec<Event> {
        if self.os_driver.is_session_finished() {
            return Vec::new();
        }

        let events = self.os_driver.update();

        for event in events.iter() {
            self.handle_event(event);
        }

        events
    }

    pub fn visible_text(&self) -> String {
        let scrollback_line_count = 0;
        self.scroll_buffer.visible_text(scrollback_line_count)
    }

    pub fn visible_slices(&self) -> Vec<TextSlice> {
        let scrollback_line_count = 0;
        self.scroll_buffer.visible_slices(scrollback_line_count)
    }

    /// Gets the cursor index.
    pub fn cursor_index(&self) -> usize {
        self.scroll_buffer.cursor_index()
    }

    /// Checks if the underlying shell session has finished.
    pub fn is_session_finished(&self) -> bool { self.os_driver.is_session_finished() }

    /// Handles a terminal event.
    fn handle_event(&mut self, event: &Event) {
        use Event::*;

        match *event {
            // FIXME: we should take into account position.
            // there are x,y values in Char
            PutCharacter { x, y, character, color, .. } => {
                self.scroll_buffer.set_cursor_xy(x, y);

                self.scroll_buffer.put_character_styled(character, Style {
                    color,
                });
            },
            ClearScreen => {
                self.scroll_buffer.clear_visible();
            },
            _ => (),
        }
    }

}

impl Action {
    pub fn apply(self, term: &mut Driver) {
        match self {
            Action::WriteText(ref text) => term.write_text(text),
            Action::Backspace => term.backspace(),
            Action::Escape => term.escape(),
            Action::CursorLeft => term.cursor_left(),
            Action::CursorRight => term.cursor_right(),
            Action::CursorUp => term.cursor_up(),
            Action::CursorDown => term.cursor_down(),
            Action::ControlCode(c) => term.control_code(c),
        }
    }
}

