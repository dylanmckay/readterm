
// FIXME: process may not stop after Drop.
// read Child docs.

#[macro_use]
extern crate log;

pub use self::color::{Color, Style};
pub use self::core::{Terminal, Settings, Action};
pub use self::event::Event;

mod color;
mod core;
mod event;
mod os;
pub mod scroll_buffer;


/// A styled set of characters.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TextSlice {
    /// The text within the slice.
    pub text: String,
    pub style: Style,
}

