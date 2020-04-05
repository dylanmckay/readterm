//! Default logic that applies to all operating systems not specifically handled.

use crate::{
    core::Settings,
    os,
    Color, Event,
};

use std::{
    io, mem,
    io::prelude::*,
    process::{Child, ChildStdin, Command, ExitStatus, Stdio},
    sync::mpsc,
};

const TEXT_COLOR: Color = Color::WHITE;

mod default_shell {
    #[cfg(unix)]
    pub use self::unix::*;
    #[cfg(windows)]
    pub use self::windows::*;

    #[allow(dead_code)]
    mod unix {
        pub const EXECUTABLE: &'static str = "sh";
        pub const ARGS: &'static [&'static str] = &["-c", "sh 2<&1"];
    }

    #[allow(dead_code)]
    mod windows {
        pub const EXECUTABLE: &'static str = "cmd";
        pub const ARGS: &'static [&'static str] = &[];
    }
}

/// An operating-system independent terminal driver.
///
/// *NOTE:* This driver does not support many features.
///
/// Features that are not supported include colors, styling, etc.
///
/// This driver operates on the standard out/err/in text streams only.
pub struct Driver {
    events: std::sync::mpsc::Receiver<manager_thread::Event>,
    shell_stdin: ChildStdin,
}

impl os::Driver for Driver {
    fn new(_: &Settings) -> Result<Self, io::Error> {
        let mut child_shell = Command::new(default_shell::EXECUTABLE)
            .args(default_shell::ARGS)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped()) // ideally stdout/stderr will be interleaved, completely on stdout.
            .spawn()?;

        let shell_stdin = mem::replace(&mut child_shell.stdin, None).unwrap();

        let rx = manager_thread::create(child_shell);

        Ok(Driver {
            events: rx,
            shell_stdin,
        })
    }

    fn write_text(&mut self, s: &str) {
        self.shell_stdin.write(s.as_bytes()).unwrap();
    }

    fn backspace(&mut self) {
        unimplemented("backspace");
    }

    fn escape(&mut self) {
        unimplemented("escape key");
    }

    fn cursor_left(&mut self) {
        unimplemented("cursor left");
    }

    fn cursor_right(&mut self) {
        unimplemented("cursor right");
    }

    fn cursor_up(&mut self) {
        unimplemented("cursor up");
    }

    fn cursor_down(&mut self) {
        unimplemented("cursor down");
    }

    fn control_code(&mut self, c: char) {
        unimplemented(format!("control code: {:?}", c));
    }

    fn signal_interrupt(&mut self) {
        unimplemented("signal interrupt");
    }

    /// Sends raw data to the underlying terminal.
    fn send_raw<S>(&mut self, s: S) where S: ToString {
        self.write_text(&s.to_string());
    }

    /// Updates the terminal.
    fn update(&mut self) -> Vec<Event> {
        let mut events = Vec::new();

        while let Ok(event) = self.events.try_recv() {
            match event {
                manager_thread::Event::WriteText { ref text } => {
                    for character in text.chars() {
                        events.push(Event::PutCharacter {
                            x: 0, // FIXME: implement
                            y: 0, // FIXME: implement
                            character,
                            bold: false,
                            italic: false,
                            underlined: false,
                            strikethrough: false,
                            color: TEXT_COLOR,
                        });
                    }
                },
                manager_thread::Event::ShellExited(exit_status) => {
                    println!("shell exited: {:?}", exit_status);
                },
            }
        }

        events
    }

    /// Checks if the underlying shell session has finished.
    fn is_session_finished(&self) -> bool {
        unimplemented!();
    }
}

mod manager_thread {
    use super::*;

    #[derive(Clone, Debug)]
    pub enum Event {
        WriteText {
            text: String,
        },
        ShellExited(ExitStatus),
    }

    /// Creates a new manager thread.
    pub fn create(mut child: Child)
        -> std::sync::mpsc::Receiver<Event> {
        let (tx, rx) = mpsc::channel();

        let shell_stdout = mem::replace(&mut child.stdout, None).unwrap();

        let _stdout_thread = {
            let tx = tx.clone();

            std::thread::spawn(move || {
                for byte in shell_stdout.bytes() {
                    let byte = byte.unwrap();
                    tx.send(Event::WriteText { text: String::from_utf8_lossy(&[byte]).to_string() }).ok();
                }
            });
        };

        let _manager_thread = std::thread::spawn(move || {
            let exit_status = child.wait().unwrap();
            tx.send(Event::ShellExited(exit_status)).ok();
        });

        rx
    }
}

fn unimplemented<S>(msg: S) where S: Into<String> {
    eprintln!("[unimplemented in OS independent driver] {}", msg.into());
}
