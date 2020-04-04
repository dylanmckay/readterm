/// A color.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

/// A style.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Style {
    /// The color of the text.
    pub color: Color,
}

impl Color {
    pub const RED: Color = Color { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 };
    pub const GREEN: Color = Color { red: 0.0, green: 1.0, blue: 0.0, alpha: 1.0 };
    pub const BLUE: Color = Color { red: 0.0, green: 0.0, blue: 1.0, alpha: 1.0 };
    pub const BLACK: Color = Color { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 };
    pub const WHITE: Color = Color { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 };

    pub fn from_packed_argb8(color: u32) -> Self {
        let alpha = (color & 0xff000000) >> 24;
        let red = (color   & 0x00ff0000) >> 16;
        let green = (color & 0x0000ff00) >> 8;
        let blue = (color  & 0x000000ff) >> 0;
        Color::from_rgba8(red as u8, green as u8, blue as u8, alpha as u8)
    }

    pub fn from_rgb8(red: u8, green: u8, blue: u8) -> Self {
        Color::from_rgba8(red, green, blue, 0xff)
    }

    pub fn from_rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color {
            red: red as f32 / 255.0,
            green: green as f32 / 255.0,
            blue: blue as f32 / 255.0,
            alpha: alpha as f32 / 255.0,
        }
    }
}
