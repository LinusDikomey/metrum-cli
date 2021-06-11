mod time;
mod ui;

use std::{sync::{Arc, Mutex}, thread, time::Duration};
use ui::Cli;
use crossterm::{event::{Event, KeyCode, KeyEvent}, terminal::{Clear, ClearType}};

fn main() {
    print!("{}", Clear(ClearType::All));
    let mut cli = Cli::new();

    let keys = Arc::new(Mutex::new(Vec::new()));

    let keys_ref = keys.clone();
    thread::spawn(move || {
        loop {
            if let Ok(evt) = crossterm::event::read() {
                match evt {
                    Event::Key(key) => keys_ref.lock().unwrap().push(key),
                    _ => ()
                }
            }
        }
    });

    'main_loop: loop {
        cli.render();
        thread::sleep(Duration::from_millis(20));
        for key in keys.lock().unwrap().drain(..) {
            match key {
                KeyEvent { code: KeyCode::Char('q'), ..} => break 'main_loop,
                _ => ()
            }
        }
    }
}
