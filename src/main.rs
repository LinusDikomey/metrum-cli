mod time;
mod ui;

use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, thread, time::Duration};
use termion::{event::Key, input::TermRead};
use ui::Cli;

fn main() {
    print!("{}", termion::clear::All);
    let mut cli = Cli::new();

    let keys = Arc::new(Mutex::new(Vec::new()));

    let keys_ref = keys.clone();
    thread::spawn(move || {
        for evt in std::io::stdin().keys() {
            if let Ok(key) = evt {
                keys_ref.lock().unwrap().push(key);
            }
        }
    });

    'main_loop: loop {
        cli.render();
        thread::sleep(Duration::from_millis(20));
        for key in keys.lock().unwrap().drain(..) {
            match key {
                Key::Char('q') => break 'main_loop,
                _ => ()
            }
        }
    }
}
