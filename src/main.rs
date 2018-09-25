extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate unicode_segmentation;
extern crate unicode_width;


mod parinfer;
mod json;
mod changes;
mod common_wrapper;

use std::env;
use std::io;
use std::io::{Read,Write};

extern crate getopts;

fn options() -> getopts::Options {
    let mut options = getopts::Options::new();
    options.optflag("h", "help", "show this help message");
    options.optopt("m", "mode", "parinfer mode (indent, paren, or smart) (default: smart)", "MODE");
    options.optflag("j", "json", "read JSON input and write JSON response");
    options
}

fn parse_args() -> getopts::Matches {
    let args: Vec<String> = env::args().collect();
    match options().parse(&args[1..]) {
        Ok(matches) => matches,
        Err(f) => { panic!(f.to_string()); }
    }
}

fn mode(matches: &getopts::Matches) -> &'static str {
    match matches.opt_str("m") {
        None => "smart",
        Some(ref s) if s == "i" || s == "indent" => "indent",
        Some(ref s) if s == "p" || s == "paren"  => "paren",
        Some(ref s) if s == "s" || s == "smart"  => "smart",
        _ => panic!("invalid mode specified for `-m`")
    }
}

pub fn main() -> io::Result<()> {
    let matches = parse_args();
    if matches.opt_present("h") {
        print!("{}", options().usage("Usage: parinfer-rust [options]"));
    } else if matches.opt_present("j") {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let request: json::Request = serde_json::from_str(&input)?;
        let answer = match common_wrapper::process(&request) {
            Ok(result) => result,
            Err(e) => json::Answer::from(e)
        };
        let output = serde_json::to_string(&answer)?;
        io::stdout().write(output.as_bytes())?;
    } else {
        let mut text = String::new();
        io::stdin().read_to_string(&mut text)?;
        let request = json::Request {
            mode: String::from(mode(&matches)),
            text,
            options: json::Options {
                changes: vec![],
                cursor_x: None,
                cursor_line: None,
                prev_text: None,
                prev_cursor_x: None,
                prev_cursor_line: None,
                force_balance: false,
                return_parens: false,
                partial_result: false,
                selection_start_line: None
            }
        };
        let answer = match common_wrapper::process(&request) {
            Ok(result) => result,
            Err(e) => json::Answer::from(e)
        };
        io::stdout().write(answer.text.as_bytes())?;
    }
    Ok(())
}
