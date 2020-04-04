use readterm::{Terminal, Settings};

fn main() {
    println!("foo");

    let mut terminal = Terminal::new(Settings::default()).unwrap();

    terminal.send_raw("echo 'foo'\n");
    dump_events(&mut terminal);

}

fn dump_events(terminal: &mut Terminal) {
    for event in wait_for_events(terminal) {
        println!("{:?}", event);
    }
}

fn wait_for_events(terminal: &mut Terminal) -> Vec<readterm::Event> {
    let mut events = Vec::new();

    // wait forever for first event
    loop {
        let new_events = terminal.update();

        if !new_events.is_empty() {
            events.extend(new_events);
            break;
        }
    }

    // keep listening to events until they stop.
    loop {
        let new_events = terminal.update();

        if !new_events.is_empty() {
            events.extend(new_events);
        } else {
            break; // stop.
        }
    }

    events
}
