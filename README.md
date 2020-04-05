# readterm

A platform-independent terminal library for Rust.

All of the other terminal libraries seem to be platform-specific.

Keeps track of terminal state and scrollback buffer, text styling, etc.

Good support for Unix platforms like Linux and Mac, very minimal (but it still compiles!) support for Windows.

## Support

* Unix (Linux and Mac)
    * Partial ANSI support
    * Colors
    * Text styles (bold, underline, italic, strikethrough)
    * Clearing the screen
    * Can render interactive vim

* Other platforms (Windows)
    * Other platforms get a basic, stdout/stdin powered terminal driver.
    * Will always compile, on any target supporting Rust's std library
    * No color support
    * No backspace support
    * No text style support

