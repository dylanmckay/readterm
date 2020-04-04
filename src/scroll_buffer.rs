use crate::{Color, TextSlice, Style};
use std::{fmt, io};

/// A scrollable terminal.
pub struct ScrollBuffer {
    settings: Settings,

    /// The lines in the buffer.
    lines: Vec<Line>,

    /// The cursor location.
    cursor: Location,
}

/// A constant-width line in the buffer.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Line {
    /// The cells in the line.
    /// All lines within a buffer will be the same length. Unused
    /// cells should be space-padded.
    pub cells: Vec<Cell>,
}

/// A cell in the grid.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Cell {
    /// What character is displayed.
    pub character: char,
    /// The style of the character.
    pub style: Style,
}

/// Scroll buffer settings.
pub struct Settings {
    /// The maximum number of columns that can be displayed at once.
    pub max_columns: usize,
    /// The maximum number of lines that can be displayed at once.
    pub max_lines: usize,
    /// The number of spaces used to render tab characters.
    pub tab_width: usize,
    /// The number of lines to keep in the history.
    pub lines_to_remember: usize,
}

/// A location relative to the top-left of the terminal.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Location {
    /// The zero-based line number relative to the top-left.
    pub line_number: usize,
    /// The zero-based column number relative to the top-left.
    pub column_number: usize,
}

impl ScrollBuffer {
    /// Creates a new scroll buffer.
    pub fn new(settings: Settings) -> Self {
        ScrollBuffer {
            // Fill the buffer with a full viewport of space-only lines.
            lines: (0..settings.max_lines).into_iter().map(|_| Line::new(&settings)).collect(),
            cursor: Location::top_left(),
            settings,
        }
    }

    /// Writes a string.
    pub fn put_str(&mut self, s: &str) {
        for c in s.chars() {
            self.put_character(c);
        }
    }

    /// Backspaces the last character.
    pub fn backspace(&mut self) {
        match self.cursor.column_number {
            0 => (),
            _ => {
                self.cursor.column_number -= 1;
                self.put_character(' ');
                self.cursor.column_number -= 1;
            },
        }
    }

    /// Clears the entire buffer, including scrollback.
    pub fn clear_everything(&mut self) {
        self.lines.clear();
        self.reset_cursor();
    }

    /// Clears all visible text.
    pub fn clear_visible(&mut self) {
        let visible_lines = self.first_visible_line_index_no_scroll()..;

        for line in self.lines[visible_lines].iter_mut() {
            *line = Line::new(&self.settings);
        }
    }

    /// Resets the cursor back to (0,0).
    pub fn reset_cursor(&mut self) {
        self.cursor = Location::top_left();
    }

    /// Sets the cursor from xy coordinates relative to the top-left corner.
    pub fn set_cursor_xy(&mut self, x: usize, y: usize) {
        self.cursor = Location { line_number: y, column_number: x };
    }

    pub fn cursor_xy(&self) -> (usize, usize) {
        (self.cursor.column_number, self.cursor.line_number)
    }

    /// Places a character into the bufer at the cursor.
    pub fn put_character(&mut self, c: char) {
        self.put_character_styled(c, Style::default())
    }

    /// Places a character into the bufer at the cursor.
    pub fn put_character_styled(&mut self, character: char, style: Style) {
        // Remove the oldest line if we've hit the scrollback limit.
        if self.lines_in_scroll_buffer() > self.settings.lines_to_remember {
            self.lines.remove(0);
        }

        match character {
            '\n' => {
                self.cursor.carriage_return();

                // Add a new line if we're reached the end of our buffer.
                if self.cursor.line_number == Location::eof(&self.settings).line_number {
                    self.add_new_whitespace_line();
                } else {
                    // Only advance the cursor line if we aren't already at the end.
                    self.cursor.line_feed();
                }
            },
            '\r' => {
                self.cursor.carriage_return();
            },
            '\t' => {
                for _ in 0..self.settings.tab_width {
                    self.put_character(' ');
                }
            },
            _ => {
                // Attempt to advance the cursor.
                // An error occurs if the end was reached.
                // In this case, add a new line and set the column back to zero.
                // No need to increment line number because the location is always relative
                // to the top left, and the cursor is already on the last line.
                if self.cursor.is_eof(&self.settings) {
                    self.add_new_whitespace_line();
                    self.cursor.carriage_return();
                } else if self.cursor.column_number >= self.settings.max_columns {
                    self.cursor.carriage_return().line_feed();
                }

                let Location { line_number, column_number } = self.cursor;

                // Replace the old character.
                self.line_at(line_number).cells[column_number] = Cell {
                    character, style,
                };
                self.cursor.column_number += 1;
            },
        }
    }

    /// The line will always be at same size as the buffer width,
    fn line_at(&mut self, line_number: usize) -> &mut Line {
        let index = self.first_visible_line_index_no_scroll() + line_number;

        let line = self.lines.get_mut(index).unwrap();
        assert_eq!(line.cells.len(), self.settings.max_columns, "line too big");
        line
    }

    /// Gets the text visible at a specified scrollback.
    fn visible_lines(&self, scrollback_line_count: usize) -> &[Line] {
        let first_index = self.first_visible_line_index(scrollback_line_count);

        &self.lines[first_index..first_index + self.settings.max_lines]
    }

    /// Gets the text visible at a specified scrollback.
    pub fn visible_cells(&self, scrollback_line_count: usize) -> Vec<Vec<Cell>> {
        self.visible_lines(scrollback_line_count).iter().map(|line| line.cells.clone()).collect()
    }

    /// Gets the visible slices.
    pub fn visible_slices(&self, scrollback_line_count: usize) -> Vec<TextSlice> {
        let mut slices = Vec::new();

        for line_cells in self.visible_cells(scrollback_line_count) {
            let mut remaining_cells = &line_cells[..];

            while !remaining_cells.is_empty() {
                let next_style = remaining_cells[0].style.clone();
                let same_style_count = remaining_cells.iter().take_while(|c| c.style == next_style).count();

                let slice_text = remaining_cells[0..same_style_count].iter().map(|c| c.character).collect();
                remaining_cells = &remaining_cells[same_style_count..];

                slices.push(TextSlice {
                    text: slice_text,
                    style: next_style,
                });
            }

            slices.push(TextSlice {
                text: "\n".to_owned(),
                style: line_cells.last().unwrap().style.clone(),
            });
        }
        slices
    }

    /// Gets the text visible at a specified scrollback.
    pub fn visible_text(&self, scrollback_line_count: usize) -> String {
        let lines: Vec<_> = self.visible_lines(scrollback_line_count)
            .iter().map(ToString::to_string).collect();
        lines.join("\n")
    }

    /// Gets the entire text, including scrollback.
    pub fn entire_text(&self) -> String {
        let lines: Vec<_> = self.lines.iter().map(ToString::to_string).collect();
        lines.join("\n")
    }

    /// Gets the cursor index relative to the top-left corner.
    pub fn cursor_index(&self) -> usize {
        (self.cursor.line_number * self.settings.max_columns) + self.cursor.column_number
    }

    fn add_new_whitespace_line(&mut self) {
        self.lines.push(Line::new(&self.settings));
    }

    fn first_visible_line_index(&self, scrollback_line_count: usize) -> usize {
        if scrollback_line_count >= self.lines_in_scroll_buffer() {
            0
        } else {
            self.first_visible_line_index_no_scroll() - scrollback_line_count
        }
    }

    fn first_visible_line_index_no_scroll(&self) -> usize {
        self.lines_in_scroll_buffer()
    }

    fn lines_in_scroll_buffer(&self) -> usize {
        self.lines.len() - self.settings.max_lines
    }
}

impl Location {
    pub fn top_left() -> Self {
        Location { line_number: 0, column_number: 0 }
    }

    /// Gets the EOF cursor location.
    pub fn eof(settings: &Settings) -> Self {
        Location {
            line_number: settings.max_lines - 1,
            column_number: settings.max_columns,
        }
    }

    pub fn carriage_return(&mut self) -> &mut Self {
        self.column_number = 0;
        self
    }

    pub fn line_feed(&mut self) -> &mut Self {
        self.line_number += 1;
        self
    }

    /// Checks if the cursor is at the very end.
    pub fn is_eof(&self, settings: &Settings) -> bool {
        *self == Location::eof(settings)
    }
}

impl io::Write for ScrollBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        self.put_str(&s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl Line {
    /// Creates a new line.
    pub fn new(settings: &Settings) -> Self {
        Line {
            cells: (0..settings.max_columns).into_iter().map(|_| Cell::default()).collect()
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for cell in self.cells.iter() {
            cell.character.fmt(fmt)?
        }
        Ok(())
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            character: ' ',
            style: Style::default(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style {
            color: Color::BLACK,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;

    const SMALL_SETTINGS: Settings = Settings {
        max_columns: 3,
        max_lines: 3,
        lines_to_remember: 2, // two lines of scrollback
        tab_width: 4,
    };

    #[test]
    fn empty_buffer_is_full_of_spaces() {
        let buffer = ScrollBuffer::new(SMALL_SETTINGS);
        assert_eq!("   \n   \n   ", buffer.entire_text());
    }

    #[test]
    fn can_fill_empty_buffer_as_expected() {
        let mut buffer = ScrollBuffer::new(SMALL_SETTINGS);

        assert_eq!("   \n   \n   ", buffer.entire_text());
        buffer.put_character('A');
        assert_eq!("A  \n   \n   ", buffer.entire_text());
        buffer.put_character('B');
        assert_eq!("AB \n   \n   ", buffer.entire_text());
        buffer.put_character('C');
        assert_eq!("ABC\n   \n   ", buffer.entire_text());
        buffer.put_character('D');
        assert_eq!("ABC\nD  \n   ", buffer.entire_text());
        buffer.put_character('E');
        assert_eq!("ABC\nDE \n   ", buffer.entire_text());
        buffer.put_character('F');
        assert_eq!("ABC\nDEF\n   ", buffer.entire_text());
        buffer.put_character('G');
        assert_eq!("ABC\nDEF\nG  ", buffer.entire_text());
        buffer.put_character('H');
        assert_eq!("ABC\nDEF\nGH ", buffer.entire_text());
        buffer.put_character('I');
        assert_eq!("ABC\nDEF\nGHI", buffer.entire_text()); // adds a new row to scrollback
        buffer.put_character('J');
        assert_eq!("DEF\nGHI\nJ  ", buffer.visible_text(0)); // does not show the oldest line anymore
    }

    #[test]
    fn correctly_handles_scrollback_last_line_but_not_eof() {
        let mut buffer = ScrollBuffer::new(SMALL_SETTINGS);
        write!(buffer, "a\nb\nc\nd").unwrap();
        assert_eq!("a  \nb  \nc  \nd  ", buffer.entire_text());
        assert_eq!("b  \nc  \nd  ", buffer.visible_text(0));
    }

    #[test]
    fn handles_new_lines() {
        let mut buffer = ScrollBuffer::new(SMALL_SETTINGS);

        write!(buffer, "h\n a\nn").unwrap();
        assert_eq!("h  \n a \nn  ", buffer.entire_text());
    }

    #[test]
    fn handles_carriage_returns() {
        let mut buffer = ScrollBuffer::new(SMALL_SETTINGS);

        write!(buffer, "h\rpa").unwrap();
        assert_eq!("pa \n   \n   ", buffer.entire_text());
    }

    #[test]
    fn throws_away_scrollback_after_limit() {
        let mut buffer = ScrollBuffer::new(SMALL_SETTINGS);

        write!(buffer, "abcdefghijklmnopqr").unwrap();
        assert_eq!("def\nghi\njkl\nmno\npqr", buffer.entire_text());
    }
}

