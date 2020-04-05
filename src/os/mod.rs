//! Operating-system specifc logic.

#[cfg(unix)] pub use self::unix as current;
#[cfg(not(unix))] pub use self::default as current;

pub mod default;

#[cfg(unix)] pub mod unix;

use crate::{core::Settings, event::Event};
use std::io;

/// An operating system specific terminal driver.
pub trait Driver : Sized {
    /// Creates a new operating-system specific driver.
    fn new(settings: &Settings) -> Result<Self, io::Error>;

    /// Writes text to the terminal.
    fn write_text(&mut self, s: &str);

    /// Backspaces the last character.
    fn backspace(&mut self);

    /// Sends the ESC character code.
    fn escape(&mut self);

    /// Moves the cursor left.
    fn cursor_left(&mut self);

    /// Moves the cursor right.
    fn cursor_right(&mut self);

    /// Moves the cursor up.
    fn cursor_up(&mut self);

    /// Moves the cursor down.
    fn cursor_down(&mut self);

    /// Sends a control code to the running process.
    fn control_code(&mut self, c: char);

    /// Sends an interrupt signal to the running program.
    fn signal_interrupt(&mut self);

    /// Sends raw data to the underlying terminal.
    fn send_raw<S>(&mut self, s: S) where S: ToString;

    /// Updates the terminal.
    fn update(&mut self) -> Vec<Event>;

    /// Checks if the underlying shell session has finished.
    fn is_session_finished(&self) -> bool;

    /// Update in a loop, blocking until events are received.
    fn update_blocking(&mut self) -> Vec<Event> {
        let mut events = Vec::new();

        // wait until we receive the first event.
        loop {
            let new_events = self.update();

            if !new_events.is_empty() {
                events.extend(new_events);
                break;
            }

            std::thread::yield_now();
        }

        // keep reading until the events stop.
        loop {
            let new_events = self.update();

            if new_events.is_empty() {
                break;
            }

            events.extend(new_events);

            std::thread::yield_now();
        }

        events
    }
}
