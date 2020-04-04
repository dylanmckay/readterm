use crate::Color;


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Event {
    PutCharacter {
        x: usize,
        y: usize,
        character: char,
        bold: bool,
        italic: bool,
        underlined: bool,
        strikethrough: bool,
        color: Color,
    },
    ClearScreen,
}
