use readterm::Settings;
use readterm::os::default::Driver;
use readterm::os::Driver as _;


fn create_driver() -> Driver {
    Driver::new(&Settings::default()).expect("failed to create driver")
}

#[test]
fn can_create_driver() {
    let _ = create_driver();
}

#[test]
fn can_echo_text() {
    let mut driver = create_driver();

    driver.write_text("echo 1\n");
    driver.write_text("exit 0\n");

    let events = driver.update_blocking();
    assert_eq!(events, build::events_for_plain_text("1\n"));
}

mod build {
    use readterm::{Color, Event};

    pub fn events_for_plain_text(s: &str) -> Vec<Event> {
        s.chars().map(|character| {
            Event::PutCharacter {
                x: 0,
                y: 0,
                character,
                bold: false,
                italic: false,
                underlined: false,
                strikethrough: false,
                color: Color::WHITE,
            }
        }).collect()
    }
}
