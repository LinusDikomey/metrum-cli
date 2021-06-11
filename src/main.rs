mod time;

use colorful::{Color, Colorful};
use crate::time::*;

fn main() {
    loop {
        print!("{}[2J", 27 as char);

        let now = MetrumDateTime::now();
        println!("{}'{} {}:{}", 
            format!("{}", now.year()).color(Color::Blue),
            format!("{:0>3}", now.day()).color(Color::LightBlue),
            format!("{:0>3}", now.minute()).color(Color::Red),
            format!("{:0>2}", now.tick()).color(Color::Yellow)
        );

        //println!("TS: {}", now.timestamp().to_string().color(Color::Green));
        std::thread::sleep(std::time::Duration::from_millis(864));
    }
}
