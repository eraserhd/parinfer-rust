extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate unicode_segmentation;
extern crate unicode_width;


mod parinfer;
mod types;
mod changes;

use std::env;
use std::io;
use std::io::{Read,Write};
use types::*;

extern crate getopts;

enum InputType {
    Json,
    Text
}

enum OutputType {
    Json,
    Text
}

fn options() -> getopts::Options {
    let mut options = getopts::Options::new();
    options.optflag("h", "help", "show this help message");
    options.optopt("", "input-format", "'json', 'text' (default: 'text')", "FMT");
    options.optopt("m", "mode", "parinfer mode (indent, paren, or smart) (default: smart)", "MODE");
    options.optopt("", "output-format", "'json', 'text' (default: 'text')", "FMT");
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

fn input_type(matches: &getopts::Matches) -> InputType {
    match matches.opt_str("input-format") {
        None => InputType::Text,
        Some(ref s) if s == "text" => InputType::Text,
        Some(ref s) if s == "json" => InputType::Json,
        Some(ref s) => panic!("unknown input format `{}`", s)
    }
}

fn output_type(matches: &getopts::Matches) -> OutputType {
    match matches.opt_str("output-format") {
        None => OutputType::Text,
        Some(ref s) if s == "text" => OutputType::Text,
        Some(ref s) if s == "json" => OutputType::Json,
        Some(ref s) => panic!("unknown output fomrat `{}`", s)
    }
}

fn request(matches: &getopts::Matches) -> io::Result<Request> {
    match input_type(matches) {
        InputType::Text => {
            let mut text = String::new();
            io::stdin().read_to_string(&mut text)?;
            Ok(Request {
                mode: String::from(mode(&matches)),
                text,
                options: Options {
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
            })
        },
        InputType::Json => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            Ok(serde_json::from_str(&input)?)
        },
    }
}

pub fn main() -> io::Result<()> {
    let matches = parse_args();
    if matches.opt_present("h") {
        print!("{}", options().usage("Usage: parinfer-rust [options]"));
        Ok(())
    } else {
        let request = request(&matches)?;
        let answer = parinfer::process(&request);
        let output = match output_type(&matches) {
            OutputType::Json => serde_json::to_string(&answer)?,
            OutputType::Text => String::from(answer.text)
        };
        io::stdout().write(output.as_bytes())?;
        Ok(())
    }
}
