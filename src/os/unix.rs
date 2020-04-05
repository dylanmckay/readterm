use crate::{
    core::Settings,
    event,
    os,
    Color,
};
use std::process::Command;
use std::{env, io, mem};

/// A Unix terminal driver.
pub struct Driver {
    /// The settings.
    settings: Settings,
    /// The underlying shell background process.
    session: rexpect::session::PtySession,
    /// Whether the underlying shell process is finished.
    session_finished: bool,
    /// The ANSI escape parser.
    parser: ransid::Console,
}

impl os::Driver for Driver {
    fn new(settings: &Settings) -> Result<Self, io::Error> {
        let session = spawn_shell(&settings);

        Ok(Driver {
            parser: create_parser(settings),
            session,
            settings: settings.clone(),
            session_finished: false,
        })
    }

    fn write_text(&mut self, s: &str) {
        self.session.send(s).unwrap();
    }

    fn backspace(&mut self) {
        self.session.send("\x08").unwrap(); // send backspace character code.
    }

    fn escape(&mut self) {
        self.session.send("\x1b").unwrap(); // send ESC character code.
    }

    fn cursor_left(&mut self) {
        self.send_raw(ansi_escapes::CursorMove::X(-1));
    }

    fn cursor_right(&mut self) {
        self.send_raw(ansi_escapes::CursorMove::X(1));
    }

    fn cursor_up(&mut self) {
        self.send_raw(ansi_escapes::CursorMove::Y(-1));
    }

    fn cursor_down(&mut self) {
        self.send_raw(ansi_escapes::CursorMove::Y(1));
    }

    fn control_code(&mut self, c: char) {
        self.session.send_control(c).expect("failed to send control code to pty");
    }

    fn signal_interrupt(&mut self) {
        self.control_code('c');
    }

    /// Sends raw data to the underlying terminal.
    fn send_raw<S>(&mut self, s: S) where S: ToString {
        self.session.send(&s.to_string()).unwrap();
    }

    /// Updates the terminal.
    fn update(&mut self) -> Vec<event::Event> {
        use rexpect::process::wait::WaitStatus::*;

        let mut events = Vec::new();

        if self.is_session_finished() {
            return events;
        }

        match self.session.process.status() {
            Some(Exited(_, _)) | None => {
                self.session_finished = true;
            },
            Some(_) => {
                while let Some(byte) = self.session.try_read_raw() {
                    // anything to appease the borrow checker.
                    let mut parser = mem::replace(&mut self.parser, create_parser(&self.settings));
                    parser.write(&[byte], |event| {
                        events.extend(self::convert_ransid_event(event))
                    });
                    self.parser = parser;
                }
            }
        }

        events
    }

    /// Checks if the underlying shell session has finished.
    fn is_session_finished(&self) -> bool { self.session_finished }
}

/// Handles a terminal event.
fn convert_ransid_event<'a>(event: ransid::Event<'a>)
    -> Vec<event::Event> {
    use ransid::Event::*;

    match event {
        // FIXME: we should take into account position.
        // there are x,y values in Char
        Char { x, y, c, color, bold, italic, underlined, strikethrough } => {
            vec![
                event::Event::PutCharacter {
                    x, y, bold, italic, underlined, strikethrough,
                    character: c,
                    color: Color::from_packed_argb8(color.as_rgb())
                }
            ]
        },
        ScreenBuffer { clear, .. } => {
            let mut events = Vec::new();

            if clear {
                events.push(event::Event::ClearScreen);
            }

            events
        },
        _ => vec![], // unimplemented event
    }
}


fn create_parser(settings: &Settings) -> ransid::Console {
    ransid::Console::new(settings.column_count, settings.line_count)
}

fn spawn_shell(settings: &Settings)
    -> rexpect::session::PtySession {

    let mut cmd = Command::new(&settings.shell);

    // FIXME: this won't exist if binaries are redistributed.
    let dir = format!("{}/../", env!("CARGO_MANIFEST_DIR"));
    cmd.current_dir(dir);

    rexpect::session::spawn_command(cmd, None)
        .expect("failed to spawn shell")
}

impl Drop for Driver {
    fn drop(&mut self) {
        if !self.session_finished {
            // We should probably do something more graceful.
            if let Err(e) = self.session.process.signal(rexpect::process::signal::Signal::SIGKILL) {
                info!("failed to kill terminal process with pid {:?}: {}",
                      self.session.process.child_pid, e);
            }
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let shell = if let Ok(shell) = env::var("SHELL") {
            shell
        } else {
            "sh".to_owned()
        };

        Settings {
            shell,
            lines_to_remember: 10_000,
            line_count: 100,
            column_count: 85,
            tab_width: 2,
        }
    }
}

